package com.xidl.idl

import com.intellij.openapi.fileTypes.LanguageFileType
import javax.swing.Icon

object IdlFileType : LanguageFileType(IdlLanguage) {
    override fun getName(): String = "IDL File"
    override fun getDescription(): String = "IDL language file"
    override fun getDefaultExtension(): String = "idl"
    override fun getIcon(): Icon = IdlIcons.FILE
}
