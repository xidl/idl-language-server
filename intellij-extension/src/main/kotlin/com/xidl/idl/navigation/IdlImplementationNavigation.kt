package com.xidl.idl.navigation

import com.intellij.codeInsight.daemon.RelatedItemLineMarkerInfo
import com.intellij.codeInsight.daemon.RelatedItemLineMarkerProvider
import com.intellij.codeInsight.navigation.NavigationGutterIconBuilder
import com.intellij.codeInsight.navigation.actions.GotoDeclarationHandler
import com.intellij.icons.AllIcons
import com.intellij.notification.NotificationGroupManager
import com.intellij.notification.NotificationType
import com.intellij.openapi.editor.Editor
import com.intellij.openapi.project.DumbService
import com.intellij.openapi.project.Project
import com.intellij.openapi.util.NotNullLazyValue
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiNameIdentifierOwner
import com.intellij.psi.PsiNamedElement
import com.intellij.psi.search.GlobalSearchScope
import com.intellij.psi.search.PsiSearchHelper
import com.intellij.psi.util.elementType

private fun notifyDebug(project: Project, message: String) {
    NotificationGroupManager.getInstance()
        .getNotificationGroup("Compiler")
        .createNotification(
            "IDL Navigation Debug",
            message,
            NotificationType.INFORMATION
        )
        .notify(project)
}

/**
 * Generates all possible casing variations of a name to support searching across different programming languages.
 * Prioritizes PascalCase for Go/Java/Rust.
 */
fun getPossibleNames(name: String, kind: SymbolKind): List<String> {
    val names = mutableListOf<String>()

    // 1. PascalCase (Standard for exported symbols in Go, types in most languages)
    val pascal = if (name.contains('_')) {
        name.split('_').joinToString("") {
            it.lowercase().replaceFirstChar { c -> c.uppercase() }
        }
    } else {
        name.replaceFirstChar { c -> c.uppercase() }
    }
    names.add(pascal)
    if (kind == SymbolKind.Interface) {
        names.add(name + "Service")
    }

    return names.distinct()
}

enum class SymbolKind {
    Interface,
    Struct,
    Enum,
    Bitmask,
    Union,
    Unknown
}

/**
 * Helper to check if a word at a specific offset in text is a declaration.
 */
fun isDeclarationAtOffset(
    text: String,
    word: String,
    offset: Int
): SymbolKind? {
    if (!word.matches(Regex("[a-zA-Z_][a-zA-Z0-9_]*"))) return null

    // Scan backward for the keyword (struct, interface, service, enum, bitmask)
    var i = offset - 1
    while (i >= 0 && text[i].isWhitespace()) {
        i--
    }
    val end = i + 1
    while (i >= 0 && (text[i].isLetterOrDigit() || text[i] == '_')) {
        i--
    }
    val start = i + 1
    if (start < end) {
        val prevWord = text.substring(start, end)
        if (prevWord == "struct") {
            return SymbolKind.Struct
        }
        if (prevWord == "interface") {
            return SymbolKind.Interface
        }
        if (prevWord == "enum") {
            return SymbolKind.Enum
        }
        if (prevWord == "bitmask") {
            return SymbolKind.Bitmask
        }
        if (prevWord == "union") {
            return SymbolKind.Union
        }
    }

    // Check if it's a method declaration: followed by '(' and not preceded by '@'
    var k = offset - 1
    while (k >= 0 && text[k].isWhitespace()) {
        k--
    }
    if (k >= 0 && text[k] == '@') {
        return null
    }

    var j = offset + word.length
    while (j < text.length && text[j].isWhitespace()) {
        j++
    }
    if (j < text.length && text[j] == '(') {
        return SymbolKind.Unknown
    }

    return null
}

/**
 * Checks if the given PsiElement inside an IDL file is a symbol declaration.
 */
fun isIdlDeclaration(element: PsiElement): Boolean {
    val file = element.containingFile ?: return false
    val langId = element.language.id
    if (langId != "IDL" && langId != "LSP" && langId != "TEXT" && !file.name.endsWith(
            ".idl"
        )
    ) return false

    val text = element.text
    val fullText = file.text ?: return false

    // If it's a leaf element (normal case)
    if (text.length < 100 && !text.contains(" ")) {
        return isDeclarationAtOffset(fullText, text, element.textOffset) != null
    }

    return false
}

/**
 * Checks if the given PsiElement is a definition of a type, class, struct, function, etc. in other target languages.
 */
fun isDefinition(
    element: PsiElement,
    kind: SymbolKind
): Boolean {

    val parent = element.parent ?: return false

    if (parent.elementType.toString() == "METHOD_DECLARATION") return false

    // If the parent is a standard IntelliJ named element, and this element is its name identifier
    if (parent is PsiNameIdentifierOwner && parent.nameIdentifier == element) {
        return true
    }

    val parentClass = parent.javaClass.name
    val parentSimpleName = parent.javaClass.simpleName

    // Robust check for Go plugin elements (support GoLand and IntelliJ Go plugin)
    if (parentClass.contains("Go")
        || parentClass.contains("com.goide")
    ) {
        // Specifically ONLY match major structural elements
        if (parentSimpleName.contains("TypeSpec") ||
            parentSimpleName.contains("Function") ||
            parentSimpleName.contains("Method") ||
            parentSimpleName.contains("Interface") ||
            parentSimpleName.contains("GoSpecTypeImpl") // interface
        ) {
            // Exclude noisy definitions
            if (parentSimpleName.contains("Var") ||
                parentSimpleName.contains("Const") ||
                parentSimpleName.contains("Field") ||
                parentSimpleName.contains("Param") ||
                parentSimpleName.contains("Receiver")
            ) {
                return false
            }
            return true
        }
        return false
    }

    if (parentClass.contains("PyClass") ||
        parentClass.contains("PyFunction") ||
        parentClass.contains("RsStruct") ||
        parentClass.contains("RsEnum") ||
        parentClass.contains("RsTrait") ||
        parentClass.contains("RsImpl") ||
        parentClass.contains("RsFunction") ||
        parentClass.contains("PsiClass") ||
        parentClass.contains("PsiMethod") ||
        parentClass.contains("KtClass") ||
        parentClass.contains("KtObjectDeclaration") ||
        parentClass.contains("KtNamedFunction")
    ) {
        // Ensure this element is actually the identifier/name of the parent
        if (parent is PsiNamedElement && parent.name == element.text) {
            return true
        }
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
 * Searches the project for definitions matching the given symbol name.
 */
fun findImplementations(
    project: Project,
    name: String,
    kind: SymbolKind
): List<PsiElement> {
    val results = mutableListOf<PsiElement>()
    val scope = GlobalSearchScope.projectScope(project)
    val possibleNames = getPossibleNames(name, kind)

    for (targetName in possibleNames) {
        PsiSearchHelper.getInstance(project).processAllFilesWithWord(
            targetName,
            scope,
            { file ->
                val langId = file.language.id
                val path = file.virtualFile?.path ?: ""
                if (langId == "IDL" || langId == "LSP" || file.name.endsWith(".idl")) return@processAllFilesWithWord true
                if (path.contains("/build/") || path.contains("/dist/") || path.contains(
                        "/out/"
                    )
                ) return@processAllFilesWithWord true

                val fileText = file.text ?: return@processAllFilesWithWord true

                // ONLY INCLUDE GENERATED FILES
                if (!fileText.contains("Code generated by xidlc") || !fileText.contains(
                        "DO NOT EDIT"
                    )
                ) {
                    return@processAllFilesWithWord true
                }

                var index = fileText.indexOf(targetName)
                while (index >= 0) {
                    val element = file.findElementAt(index)
                    if (element != null && element.text == targetName) {
                        if (isDefinition(element, kind)) {
                            val decl = element.parent
                            if (decl != null && !results.contains(decl)) {
                                results.add(decl)
                            }
                        }
                    }
                    if (results.size > 20) return@processAllFilesWithWord false
                    index = fileText.indexOf(targetName, index + 1)
                }
                true
            },
            true
        )
        // If we found a PascalCase match (likely Go exported symbol), we can stop early if we have enough
        if (targetName[0].isUpperCase() && results.size >= 2) break
    }

    // Final filtering and prioritization
    val prioritized = results.filter {
        val path = it.containingFile.virtualFile?.path ?: ""
        !path.contains("_test.go") && !path.contains("Test.java")
    }.sortedBy {
        // Prefer PascalCase matches for Go/Java
        val elementName = (it as? PsiNamedElement)?.name ?: ""
        if (elementName.isNotEmpty() && elementName[0].isUpperCase()) 0 else 1
    }

    return prioritized.ifEmpty { results }
}

/**
 * Shows gutter line markers in IDL files.
 */
class IdlImplementationLineMarkerProvider : RelatedItemLineMarkerProvider() {
    override fun collectNavigationMarkers(
        element: PsiElement,
        result: MutableCollection<in RelatedItemLineMarkerInfo<*>>
    ) {
        if (DumbService.isDumb(element.project)) return

        val file = element.containingFile ?: return
        if (!file.name.endsWith(".idl")) return

        val langId = element.language.id

        // If it's a normal IDL PSI (tokenized)
        if (langId == "IDL" || langId == "LSP") {
            if (isIdlDeclaration(element)) {
                addMarker(element, element.text, result)
            }
            return
        }

        // If it's PlainText
        if (langId == "TEXT" && element.parent == null) {
            val fullText = element.text
            val regex =
                Regex("\\b(struct|interface|service|enum|bitmask)\\s+([a-zA-Z_][a-zA-Z0-9_]*)")
            regex.findAll(fullText).forEach { match ->
                val name = match.groups[2]?.value ?: return@forEach
                val range = match.groups[2]?.range ?: return@forEach
                val leaf = element.findElementAt(range.first)
                if (leaf != null) {
                    addMarker(leaf, name, result)
                }
            }

            val methodRegex = Regex("\\n\\s*([a-zA-Z_][a-zA-Z0-9_]*)\\s*\\(")
            methodRegex.findAll(fullText).forEach { match ->
                val name = match.groups[1]?.value ?: return@forEach
                val range = match.groups[1]?.range ?: return@forEach
                if (name == "struct" || name == "interface" || name == "service" || name == "enum" || name == "bitmask") return@forEach
                val leaf = element.findElementAt(range.first)
                if (leaf != null) {
                    addMarker(leaf, name, result)
                }
            }
        }
    }

    private fun addMarker(
        anchor: PsiElement,
        name: String,
        result: MutableCollection<in RelatedItemLineMarkerInfo<*>>
    ) {
        val builder =
            NavigationGutterIconBuilder.create(AllIcons.Gutter.ImplementedMethod)
                .setTooltipText("Navigate to implementation of '$name'")
                .setEmptyPopupText("No implementations found for '$name'")
                .setTargets(NotNullLazyValue.lazy {
                    findImplementations(
                        anchor.project,
                        name,
                        SymbolKind.Unknown
                    )
                })

        result.add(builder.createLineMarkerInfo(anchor))
    }
}

/**
 * Handles F12 / Ctrl+Click.
 */
class IdlGotoDeclarationHandler : GotoDeclarationHandler {
    override fun getGotoDeclarationTargets(
        sourceElement: PsiElement?,
        offset: Int,
        editor: Editor?
    ): Array<PsiElement>? {
        if (sourceElement == null || editor == null || DumbService.isDumb(
                sourceElement.project
            )
        ) return null

        val file = sourceElement.containingFile ?: return null
        if (!file.name.endsWith(".idl")) return null

        val fullText = file.text ?: return null

        var start = offset
        while (start > 0 && (fullText[start - 1].isLetterOrDigit() || fullText[start - 1] == '_')) {
            start--
        }
        var end = offset
        while (end < fullText.length && (fullText[end].isLetterOrDigit() || fullText[end] == '_')) {
            end++
        }

        if (start >= end) return null
        val name = fullText.substring(start, end)

        var symbolKind = isDeclarationAtOffset(fullText, name, start)
        if (symbolKind == null) {
            return null;
        }

        val targets =
            findImplementations(sourceElement.project, name, symbolKind)

        return if (targets.isEmpty()) null else targets.toTypedArray()
    }
}
