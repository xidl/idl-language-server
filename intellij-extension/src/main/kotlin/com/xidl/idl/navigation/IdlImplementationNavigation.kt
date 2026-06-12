package com.xidl.idl.navigation

import com.intellij.codeInsight.daemon.RelatedItemLineMarkerInfo
import com.intellij.codeInsight.daemon.RelatedItemLineMarkerProvider
import com.intellij.codeInsight.navigation.NavigationGutterIconBuilder
import com.intellij.codeInsight.navigation.actions.GotoDeclarationHandler
import com.intellij.icons.AllIcons
import com.intellij.openapi.editor.Editor
import com.intellij.openapi.project.DumbService
import com.intellij.openapi.project.Project
import com.intellij.openapi.util.NotNullLazyValue
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiNameIdentifierOwner
import com.intellij.psi.PsiNamedElement
import com.intellij.psi.search.GlobalSearchScope
import com.intellij.psi.search.PsiSearchHelper

/**
 * Generates all possible casing variations of a name (camelCase, PascalCase, snake_case, original)
 * to support searching across different programming languages.
 */
fun getPossibleNames(name: String): List<String> {
    val names = mutableListOf(name)
    
    // Support common generated code suffixes
    names.add(name + "Server")
    names.add(name + "Client")
    names.add(name + "Impl")
    names.add("I" + name)

    if (name.contains('_')) {
        val parts = name.split('_')
        if (parts.isNotEmpty()) {
            // camelCase
            val camel = StringBuilder(parts[0].lowercase())
            for (i in 1 until parts.size) {
                val part = parts[i]
                if (part.isNotEmpty()) {
                    camel.append(part.substring(0, 1).uppercase())
                    camel.append(part.substring(1).lowercase())
                }
            }
            names.add(camel.toString())

            // PascalCase
            val pascal = StringBuilder()
            for (part in parts) {
                if (part.isNotEmpty()) {
                    pascal.append(part.substring(0, 1).uppercase())
                    pascal.append(part.substring(1).lowercase())
                }
            }
            names.add(pascal.toString())
        }
    } else {
        val snake = name.replace(Regex("([a-z])([A-Z])"), "$1_$2").lowercase()
        if (snake != name) {
            names.add(snake)
        }
        if (name.isNotEmpty()) {
            val firstChar = name[0]
            if (firstChar.isLowerCase()) {
                names.add(name.substring(0, 1).uppercase() + name.substring(1))
            } else if (firstChar.isUpperCase()) {
                names.add(name.substring(0, 1).lowercase() + name.substring(1))
            }
        }
    }

    // Final pass for all variations with suffixes
    val allNames = names.toMutableList()
    for (n in names) {
        allNames.add(n + "Server")
        allNames.add(n + "Impl")
    }

    return allNames.distinct()
}

/**
 * Checks if the given PsiElement inside an IDL file is a symbol declaration (e.g. struct, interface, service, enum, bitmask name, or method name).
 */
fun isIdlDeclaration(element: PsiElement): Boolean {
    val file = element.containingFile ?: return false
    // Support IDL files even if another plugin is providing a coarse PSI (e.g. PlainText)
    if (element.language.id != "IDL" && !file.name.endsWith(".idl")) return false
    
    val text = element.text
    // A declaration identifier should be a single word, not a full line or containing spaces
    if (text.isEmpty() || text.length > 100 || text.any { it.isWhitespace() }) return false
    if (!text.matches(Regex("[a-zA-Z_][a-zA-Z0-9_]*"))) return false
    
    val fileText = file.text ?: return false
    val offset = element.textOffset
    if (offset < 0) return false

    // Scan backward for the keyword (struct, interface, service, enum, bitmask)
    var i = offset - 1
    while (i >= 0 && fileText[i].isWhitespace()) {
        i--
    }
    val end = i + 1
    while (i >= 0 && (fileText[i].isLetterOrDigit() || fileText[i] == '_')) {
        i--
    }
    val start = i + 1
    if (start < end) {
        val prevWord = fileText.substring(start, end)
        if (prevWord == "struct" || prevWord == "interface" || prevWord == "service" || prevWord == "enum" || prevWord == "bitmask") {
            return true
        }
    }

    // Check if it's a method declaration: followed by '(' and not preceded by '@' (which would be an annotation)
    var k = offset - 1
    while (k >= 0 && fileText[k].isWhitespace()) {
        k--
    }
    if (k >= 0 && fileText[k] == '@') {
        return false
    }

    var j = offset + text.length
    while (j < fileText.length && fileText[j].isWhitespace()) {
        j++
    }
    if (j < fileText.length && fileText[j] == '(') {
        return true
    }

    return false
}

/**
 * Checks if the given PsiElement is a definition of a type, class, struct, function, etc. in other target languages.
 */
fun isDefinition(element: PsiElement): Boolean {
    val parent = element.parent ?: return false

    // If the parent is a standard IntelliJ named element, and this element is its name identifier
    if (parent is PsiNameIdentifierOwner && parent.nameIdentifier == element) {
        return true
    }

    // Check by class name to support external language plugins dynamically without direct dependency
    val parentClass = parent.javaClass.name
    val parentSimpleName = parent.javaClass.simpleName
    
    // Robust check for Go plugin elements (support GoLand and IntelliJ Go plugin)
    if (parentClass.contains("Go") || parentClass.contains("com.goide")) {
        if (parentSimpleName.contains("TypeSpec") ||
            parentSimpleName.contains("Function") ||
            parentSimpleName.contains("Method") ||
            parentSimpleName.contains("Interface") ||
            parentSimpleName.contains("Spec") ||
            parentSimpleName.contains("Definition") ||
            parentSimpleName.contains("Declaration") ||
            parentSimpleName.contains("Type") ||
            parentSimpleName.contains("Var") ||
            parentSimpleName.contains("Const")
        ) {
            return true
        }
    }

    if (parentClass.contains("PyClass") ||
        parentClass.contains("PyFunction") ||
        parentClass.contains("PyTargetExpression") ||
        parentClass.contains("RsStruct") ||
        parentClass.contains("RsEnum") ||
        parentClass.contains("RsTrait") ||
        parentClass.contains("RsImpl") ||
        parentClass.contains("RsFunction") ||
        parentClass.contains("RsType") ||
        parentClass.contains("PsiClass") ||
        parentClass.contains("PsiMethod") ||
        parentClass.contains("KtClass") ||
        parentClass.contains("KtObjectDeclaration") ||
        parentClass.contains("KtNamedFunction") ||
        parentClass.contains("KtProperty") ||
        parentClass.contains("KtConstructor")
    ) {
        // Ensure this element is actually the identifier/name of the parent
        if (parent is PsiNamedElement && parent.name == element.text) {
            return true
        }
        // Fallback for custom AST node names that don't fully implement PsiNamedElement
        try {
            val nameMethod = parent.javaClass.getMethod("getName")
            val name = nameMethod.invoke(parent) as? String
            if (name == element.text) {
                return true
            }
        } catch (e: Exception) {
            // ignore
        }
    }
    return false
}

/**
 * Searches the project (excluding IDL files themselves) for type/class/struct/method definitions matching the given symbol name
 * or its casing variations (camelCase, PascalCase, snake_case).
 */
fun findImplementations(project: Project, name: String): List<PsiElement> {
    val results = mutableListOf<PsiElement>()
    val scope = GlobalSearchScope.allScope(project)
    val possibleNames = getPossibleNames(name)

    for (targetName in possibleNames) {
        PsiSearchHelper.getInstance(project).processAllFilesWithWord(
            targetName,
            scope,
            { file ->
                // Skip searching in IDL files to only find target implementations (Go, Python, Rust, etc.)
                if (file.language.id == "IDL") return@processAllFilesWithWord true

                val fileText = file.text
                var index = fileText.indexOf(targetName)
                while (index >= 0) {
                    val element = file.findElementAt(index)
                    if (element != null && element.text == targetName) {
                        if (isDefinition(element)) {
                            val decl = element.parent
                            if (decl != null && !results.contains(decl)) {
                                results.add(decl)
                            }
                        }
                    }
                    index = fileText.indexOf(targetName, index + 1)
                }
                true // Continue searching
            },
            true // Case-sensitive
        )
    }
    return results
}

/**
 * Shows gutter line markers in IDL files next to struct, service, enum definitions.
 * Clicking the marker allows navigating to matching Go/Python/Rust/Java implementations.
 */
class IdlImplementationLineMarkerProvider : RelatedItemLineMarkerProvider() {
    override fun collectNavigationMarkers(
        element: PsiElement,
        result: MutableCollection<in RelatedItemLineMarkerInfo<*>>
    ) {
        if (DumbService.isDumb(element.project)) return
        if (!isIdlDeclaration(element)) return

        val name = element.text
        val builder = NavigationGutterIconBuilder.create(AllIcons.Gutter.ImplementedMethod)
            .setTooltipText("Navigate to Go/Python/Rust implementation")
            .setEmptyPopupText("No implementations found")
            .setTargets(NotNullLazyValue.lazy {
                findImplementations(element.project, name)
            })

        result.add(builder.createLineMarkerInfo(element))
    }
}

/**
 * Handles F12 / Ctrl+Click (Go to Declaration/Definition) on IDL declarations to jump directly to matching implementations.
 */
class IdlGotoDeclarationHandler : GotoDeclarationHandler {
    override fun getGotoDeclarationTargets(
        sourceElement: PsiElement?,
        offset: Int,
        editor: Editor?
    ): Array<PsiElement>? {
        if (sourceElement == null || DumbService.isDumb(sourceElement.project)) return null
        if (!isIdlDeclaration(sourceElement)) return null

        val name = sourceElement.text
        val targets = findImplementations(sourceElement.project, name)
        if (targets.isEmpty()) return null
        return targets.toTypedArray()
    }
}
