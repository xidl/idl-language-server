package com.xidl.idl.lsp

import com.intellij.execution.configurations.GeneralCommandLine
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.platform.lsp.api.LspServerSupportProvider
import com.intellij.platform.lsp.api.ProjectWideLspServerDescriptor
import com.xidl.idl.IdlFileType

class IdlLspServerSupportProvider : LspServerSupportProvider {
    override fun fileOpened(
        project: Project,
        file: VirtualFile,
        serverStarter: LspServerSupportProvider.LspServerStarter
    ) {
        if (file.fileType == IdlFileType) {
            serverStarter.ensureServerStarted(
                IdlLspServerDescriptor(
                    project,
                    "IDL"
                )
            )
        }
    }
}

class IdlLspServerDescriptor(project: Project, presentableName: String) :
    ProjectWideLspServerDescriptor(project, presentableName) {
    override fun isSupportedFile(file: VirtualFile) =
        file.fileType == IdlFileType

    override fun createCommandLine(): GeneralCommandLine {
        return GeneralCommandLine("idl-language-server")
    }
}
