plugins {
    id("java")
    id("org.jetbrains.kotlin.jvm") version "1.9.23"
    id("org.jetbrains.intellij.platform") version "2.0.1"
}

group = providers.gradleProperty("pluginGroup").get()
version = providers.gradleProperty("pluginVersion").get().substringBefore(" #").trim()

kotlin {
    jvmToolchain(17)
}

java {
    toolchain {
        languageVersion = JavaLanguageVersion.of(17)
    }
}

repositories {
    mavenCentral()
    intellijPlatform {
        defaultRepositories()
    }
}

dependencies {
    intellijPlatform {
        intellijIdeaUltimate(providers.gradleProperty("platformVersion"))
        // We need the LSP plugin bundled in Ultimate
        // Actually, in modern platform versions, LSP is part of the platform/bundled, but we might not need to declare it as a bundled plugin if it's intrinsic.
        // Let's try declaring it just in case, or we can just rely on the platform.
        // Wait, LSP is not a separate bundled plugin in 241, it's just in the platform. Let's omit `bundledPlugin("com.intellij.platform.lsp")` unless needed.
        instrumentationTools()
        pluginVerifier()
        zipSigner()
    }
}

intellijPlatform {
    pluginConfiguration {
        version = providers.gradleProperty("pluginVersion")
        ideaVersion {
            sinceBuild = providers.gradleProperty("pluginSinceBuild")
            untilBuild = providers.gradleProperty("pluginUntilBuild")
        }
    }
}
