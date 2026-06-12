package com.xidl.idl.lsp

import com.intellij.execution.configurations.GeneralCommandLine
import com.intellij.ide.plugins.PluginManagerCore
import com.intellij.openapi.extensions.PluginId
import com.intellij.openapi.project.Project
import com.intellij.openapi.util.SystemInfo
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.platform.lsp.api.LspServerSupportProvider
import com.intellij.platform.lsp.api.ProjectWideLspServerDescriptor
import com.xidl.idl.IdlFileType
import java.nio.file.Files

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
        val pluginId = PluginId.getId("com.xidl.idl")
        val plugin = PluginManagerCore.getPlugin(pluginId)

        val binName = if (SystemInfo.isWindows) "idl-language-server.exe" else "idl-language-server"
        val osDir = when {
            SystemInfo.isWindows -> "windows"
            SystemInfo.isMac -> "mac"
            SystemInfo.isLinux -> "linux"
            else -> "linux"
        }

        val bundledPath = plugin?.pluginPath?.resolve("bin")?.resolve(osDir)?.resolve(binName)

        if (bundledPath != null && Files.exists(bundledPath)) {
            return GeneralCommandLine(bundledPath.toString())
        }

        return GeneralCommandLine("idl-language-server")
    }
}
