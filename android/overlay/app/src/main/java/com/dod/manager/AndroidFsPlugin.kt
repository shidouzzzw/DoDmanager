// SPDX-License-Identifier: GPL-3.0-or-later
package com.dod.manager

import android.app.Activity
import android.content.ClipData
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.os.StatFs
import android.provider.DocumentsContract
import android.provider.Settings
import androidx.activity.result.ActivityResult
import androidx.core.content.FileProvider
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsControllerCompat
import androidx.documentfile.provider.DocumentFile
import app.tauri.annotation.ActivityCallback
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin
import org.json.JSONArray
import java.io.File

@InvokeArg
internal class UriArgs {
    lateinit var uri: String
}

@InvokeArg
internal class ParentArgs {
    lateinit var parentUri: String
    lateinit var name: String
    var mime: String? = null
}

@InvokeArg
internal class RenameArgs {
    lateinit var uri: String
    lateinit var newName: String
}

@InvokeArg
internal class CopyArgs {
    lateinit var sourceUri: String
    lateinit var destParentUri: String
    lateinit var destName: String
}

@InvokeArg
internal class TransferArgs {
    lateinit var uri: String
    lateinit var destPath: String
}

@InvokeArg
internal class ImportArgs {
    lateinit var srcPath: String
    lateinit var destUri: String
}

@InvokeArg
internal class SystemBarsArgs {
    var dark: Boolean = false
}

@InvokeArg
internal class PathArgs {
    var path: String? = null
}

@InvokeArg
internal class PathsArgs {
    var paths: List<String> = emptyList()
}

@InvokeArg
internal class OpenArgs {
    lateinit var path: String
    var mime: String? = null
}

/**
 * Android tarafındaki Tauri eklentisi. SAF (Storage Access Framework) işlemleri,
 * paylaşım, "Birlikte aç" ve dış uygulamalardan gelen dosyalar burada yönetilir.
 * Kalıcı URI izinleri Android'in kendi izin deposunda tutulur.
 */
@TauriPlugin
class AndroidFsPlugin(private val activity: Activity) : Plugin(activity) {

    companion object {
        private const val TAG = "AndroidFs"
        private const val PICK_DIRECTORY = 4101
        private const val PREF = "dod_saf_prefs"
        private const val INCOMING_KEY = "incoming_queue"

        /** MainActivity tarafından doldurulur; eklenti örneği olmadan da erişilir. */
        private val pendingIncoming = mutableListOf<Map<String, Any>>()

        fun captureIntent(intent: Intent?) {
            if (intent == null) return
            try {
                when (intent.action) {
                    Intent.ACTION_VIEW -> {
                        intent.data?.let { pendingIncoming.add(describe(it, intent.type)) }
                    }
                    Intent.ACTION_SEND -> {
                        val uri = @Suppress("DEPRECATION") intent.getParcelableExtra<Uri>(Intent.EXTRA_STREAM)
                        uri?.let { pendingIncoming.add(describe(it, intent.type)) }
                    }
                    Intent.ACTION_SEND_MULTIPLE -> {
                        val list = @Suppress("DEPRECATION")
                        intent.getParcelableArrayListExtra<Uri>(Intent.EXTRA_STREAM)
                        list?.forEach { pendingIncoming.add(describe(it, intent.type)) }
                    }
                }
            } catch (_: Exception) {
                // Bozuk intent uygulamayı düşürmemeli.
            }
        }

        private fun describe(uri: Uri, mime: String?): Map<String, Any> {
            return mapOf(
                "uri" to uri.toString(),
                "name" to (uri.lastPathSegment ?: "paylasilan"),
                "mime" to (mime ?: "application/octet-stream"),
                "size" to 0L
            )
        }
    }

    // ---------------------------------------------------------------------
    // Yardımcılar
    // ---------------------------------------------------------------------

    /** Belge URI'sinden, alt öğeleri listelenebilen ağaç belgesi üretir. */
    private fun treeDocument(uri: Uri): DocumentFile? {
        val authority = uri.authority ?: return null
        val documentId = runCatching { DocumentsContract.getDocumentId(uri) }
            .getOrElse { runCatching { DocumentsContract.getTreeDocumentId(uri) }.getOrNull() }
            ?: return null
        val treeUri = DocumentsContract.buildTreeDocumentUri(authority, documentId)
        return DocumentFile.fromTreeUri(activity, treeUri)
    }

    private fun fileProviderUri(path: String): Uri {
        return FileProvider.getUriForFile(activity, "${activity.packageName}.fileprovider", File(path))
    }

    private fun shareUri(path: String): Uri {
        return if (path.startsWith("content://")) Uri.parse(path) else fileProviderUri(path)
    }

    private fun mapEntry(file: DocumentFile): Map<String, Any> {
        val name = file.name ?: file.uri.lastPathSegment ?: ""
        return mapOf(
            "uri" to file.uri.toString(),
            "name" to name,
            "isDir" to file.isDirectory,
            "size" to if (file.isDirectory) 0L else file.length(),
            "modified" to (file.lastModified() / 1000L),
            "readOnly" to !file.canWrite()
        )
    }

    private fun statMap(uri: Uri): Map<String, Any>? {
        val file = DocumentFile.fromSingleUri(activity, uri) ?: return null
        // Ağaç kökü için ad DocumentsContract'tan gelir.
        val name = file.name ?: runCatching {
            DocumentsContract.getTreeDocumentId(uri).substringAfterLast(':').substringAfterLast('/')
        }.getOrDefault("")
        return mapOf(
            "uri" to uri.toString(),
            "name" to name,
            "isDir" to (file.isDirectory || DocumentsContract.getDocumentId(uri).length <= 0),
            "size" to if (file.isDirectory) 0L else file.length(),
            "modified" to (file.lastModified() / 1000L),
            "readOnly" to !file.canWrite()
        )
    }

    private fun copyStreams(source: Uri, destination: Uri): Long {
        val input = activity.contentResolver.openInputStream(source)
            ?: throw IllegalStateException("Kaynak açılamadı")
        val output = activity.contentResolver.openOutputStream(destination, "wt")
            ?: throw IllegalStateException("Hedef açılamadı")
        input.use { ins -> output.use { outs -> return ins.copyTo(outs, 128 * 1024) } }
    }

    private fun fail(invoke: Invoke, message: String) {
        invoke.reject(message, "SAF_ERROR")
    }

    // ---------------------------------------------------------------------
    // Klasör seçimi ve izinler
    // ---------------------------------------------------------------------

    @Command
    fun pickDirectory(invoke: Invoke) {
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT_TREE).apply {
            addFlags(
                Intent.FLAG_GRANT_READ_URI_PERMISSION or
                    Intent.FLAG_GRANT_WRITE_URI_PERMISSION or
                    Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION or
                    Intent.FLAG_GRANT_PREFIX_URI_PERMISSION
            )
        }
        startActivityForResult(invoke, intent, "pickDirectoryResult")
    }

    @ActivityCallback
    private fun pickDirectoryResult(invoke: Invoke, result: ActivityResult) {
        if (result.resultCode != Activity.RESULT_OK) {
            fail(invoke, "Klasör seçimi iptal edildi")
            return
        }
        val uri = result.data?.data
        if (uri == null) {
            fail(invoke, "Seçilen klasör okunamadı")
            return
        }
        val flags = (result.data?.flags ?: 0).and(
            Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION
        )
        try {
            activity.contentResolver.takePersistableUriPermission(
                uri,
                flags or Intent.FLAG_GRANT_READ_URI_PERMISSION
            )
        } catch (error: SecurityException) {
            fail(invoke, "Kalıcı izin alınamadı: ${error.message}")
            return
        }
        val name = DocumentFile.fromTreeUri(activity, uri)?.name ?: "Seçilen klasör"
        invoke.resolveObject(mapOf("uri" to uri.toString(), "name" to name))
    }

    @Command
    fun releasePermission(invoke: Invoke) {
        val args = invoke.parseArgs(UriArgs::class.java)
        try {
            activity.contentResolver.releasePersistableUriPermission(
                Uri.parse(args.uri),
                Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION
            )
            invoke.resolveObject(mapOf("released" to true))
        } catch (error: SecurityException) {
            fail(invoke, "İzin bırakılamadı: ${error.message}")
        }
    }

    @Command
    fun hasAllFilesAccess(invoke: Invoke) {
        val granted = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            Environment.isExternalStorageManager()
        } else {
            activity.checkSelfPermission(android.Manifest.permission.WRITE_EXTERNAL_STORAGE) ==
                android.content.pm.PackageManager.PERMISSION_GRANTED
        }
        invoke.resolveObject(mapOf("granted" to granted))
    }

    @Command
    fun openAllFilesAccessSettings(invoke: Invoke) {
        try {
            val intent = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
                Intent(
                    Settings.ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION,
                    Uri.parse("package:${activity.packageName}")
                )
            } else {
                Intent(
                    Settings.ACTION_APPLICATION_DETAILS_SETTINGS,
                    Uri.parse("package:${activity.packageName}")
                )
            }
            intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            activity.startActivity(intent)
            invoke.resolveObject(mapOf("opened" to true))
        } catch (error: Exception) {
            fail(invoke, "Ayarlar açılamadı: ${error.message}")
        }
    }

    @Command
    fun openAppSettings(invoke: Invoke) {
        try {
            val intent = Intent(
                Settings.ACTION_APPLICATION_DETAILS_SETTINGS,
                Uri.parse("package:${activity.packageName}")
            ).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            activity.startActivity(intent)
            invoke.resolveObject(mapOf("opened" to true))
        } catch (error: Exception) {
            fail(invoke, "Ayarlar açılamadı: ${error.message}")
        }
    }

    // ---------------------------------------------------------------------
    // SAF üzerinde dosya işlemleri
    // ---------------------------------------------------------------------

    @Command
    fun listChildren(invoke: Invoke) {
        val args = invoke.parseArgs(UriArgs::class.java)
        val dir = treeDocument(Uri.parse(args.uri))
        if (dir == null) {
            fail(invoke, "Klasör açılamadı")
            return
        }
        if (!dir.isDirectory) {
            fail(invoke, "Bu öğe bir klasör değil")
            return
        }
        val entries = try {
            dir.listFiles().map { mapEntry(it) }
        } catch (error: Exception) {
            fail(invoke, "Klasör okunamadı: ${error.message}")
            return
        }
        invoke.resolveObject(mapOf("entries" to entries))
    }

    @Command
    fun stat(invoke: Invoke) {
        val args = invoke.parseArgs(UriArgs::class.java)
        val uri = Uri.parse(args.uri)
        val entry = statMap(uri) ?: treeDocument(uri)?.let { mapEntry(it) }
        if (entry == null) {
            fail(invoke, "Öğe bulunamadı")
        } else {
            invoke.resolveObject(entry)
        }
    }

    @Command
    fun createDirectory(invoke: Invoke) {
        val args = invoke.parseArgs(ParentArgs::class.java)
        val parent = treeDocument(Uri.parse(args.parentUri))
        if (parent == null) {
            fail(invoke, "Hedef klasör açılamadı")
            return
        }
        val created = parent.createDirectory(args.name)
        if (created == null) {
            fail(invoke, "Klasör oluşturulamadı (ad çakışıyor olabilir)")
        } else {
            invoke.resolveObject(mapOf("uri" to created.uri.toString(), "name" to (created.name ?: args.name)))
        }
    }

    @Command
    fun createFile(invoke: Invoke) {
        val args = invoke.parseArgs(ParentArgs::class.java)
        val parent = treeDocument(Uri.parse(args.parentUri))
        if (parent == null) {
            fail(invoke, "Hedef klasör açılamadı")
            return
        }
        val created = parent.createFile(args.mime ?: "application/octet-stream", args.name)
        if (created == null) {
            fail(invoke, "Dosya oluşturulamadı (ad çakışıyor olabilir)")
        } else {
            invoke.resolveObject(mapOf("uri" to created.uri.toString(), "name" to (created.name ?: args.name)))
        }
    }

    @Command
    fun renameDocument(invoke: Invoke) {
        val args = invoke.parseArgs(RenameArgs::class.java)
        try {
            val renamed = DocumentsContract.renameDocument(
                activity.contentResolver,
                Uri.parse(args.uri),
                args.newName
            )
            if (renamed == null) {
                fail(invoke, "Yeniden adlandırma desteklenmiyor")
            } else {
                invoke.resolveObject(mapOf("uri" to renamed.toString(), "name" to args.newName))
            }
        } catch (error: Exception) {
            fail(invoke, "Yeniden adlandırılamadı: ${error.message}")
        }
    }

    @Command
    fun deleteDocument(invoke: Invoke) {
        val args = invoke.parseArgs(UriArgs::class.java)
        try {
            val deleted = DocumentsContract.deleteDocument(
                activity.contentResolver,
                Uri.parse(args.uri)
            )
            if (deleted) {
                invoke.resolveObject(mapOf("deleted" to true))
            } else {
                fail(invoke, "Öğe silinemedi")
            }
        } catch (error: Exception) {
            fail(invoke, "Silinemedi: ${error.message}")
        }
    }

    @Command
    fun copyDocument(invoke: Invoke) {
        val args = invoke.parseArgs(CopyArgs::class.java)
        copyOrMove(invoke, args, deleteSource = false)
    }

    @Command
    fun moveDocument(invoke: Invoke) {
        val args = invoke.parseArgs(CopyArgs::class.java)
        copyOrMove(invoke, args, deleteSource = true)
    }

    private fun copyOrMove(invoke: Invoke, args: CopyArgs, deleteSource: Boolean) {
        val parent = treeDocument(Uri.parse(args.destParentUri))
        if (parent == null) {
            fail(invoke, "Hedef klasör açılamadı")
            return
        }
        val source = DocumentFile.fromSingleUri(activity, Uri.parse(args.sourceUri))
        if (source == null) {
            fail(invoke, "Kaynak açılamadı")
            return
        }
        if (source.isDirectory) {
            fail(invoke, "Klasörler tek tek kopyalanır")
            return
        }
        val target = parent.createFile(source.type ?: "application/octet-stream", args.destName)
        if (target == null) {
            fail(invoke, "Hedef dosya oluşturulamadı")
            return
        }
        try {
            val written = copyStreams(source.uri, target.uri)
            if (deleteSource) {
                DocumentsContract.deleteDocument(activity.contentResolver, source.uri)
            }
            invoke.resolveObject(
                mapOf(
                    "uri" to target.uri.toString(),
                    "name" to (target.name ?: args.destName),
                    "size" to written
                )
            )
        } catch (error: Exception) {
            runCatching { target.delete() }
            fail(invoke, "Kopyalama başarısız: ${error.message}")
        }
    }

    @Command
    fun exportTo(invoke: Invoke) {
        val args = invoke.parseArgs(TransferArgs::class.java)
        try {
            val target = File(args.destPath)
            target.parentFile?.mkdirs()
            val input = activity.contentResolver.openInputStream(Uri.parse(args.uri))
                ?: throw IllegalStateException("Kaynak açılamadı")
            input.use { ins ->
                target.outputStream().use { outs -> ins.copyTo(outs, 128 * 1024) }
            }
            invoke.resolveObject(mapOf("size" to target.length()))
        } catch (error: Exception) {
            fail(invoke, "Dışa aktarılamadı: ${error.message}")
        }
    }

    @Command
    fun importFrom(invoke: Invoke) {
        val args = invoke.parseArgs(ImportArgs::class.java)
        try {
            val source = File(args.srcPath)
            val output = activity.contentResolver.openOutputStream(Uri.parse(args.destUri), "wt")
                ?: throw IllegalStateException("Hedef açılamadı")
            source.inputStream().use { ins ->
                output.use { outs -> ins.copyTo(outs, 128 * 1024) }
            }
            invoke.resolveObject(mapOf("size" to source.length()))
        } catch (error: Exception) {
            fail(invoke, "İçe aktarılamadı: ${error.message}")
        }
    }

    // ---------------------------------------------------------------------
    // Depolama ve sistem
    // ---------------------------------------------------------------------

    @Command
    fun externalRoots(invoke: Invoke) {
        val roots = mutableListOf<Map<String, Any>>()
        val primary = Environment.getExternalStorageDirectory()
        roots.add(
            mapOf(
                "path" to primary.absolutePath,
                "label" to "Dahili depolama",
                "removable" to false
            )
        )
        activity.getExternalFilesDirs(null).forEachIndexed { index, dir ->
            if (dir == null) return@forEachIndexed
            val rootPath = dir.absolutePath.substringBefore("/Android/data")
            if (roots.any { it["path"] == rootPath }) return@forEachIndexed
            val removable = runCatching { Environment.isExternalStorageRemovable(File(rootPath)) }
                .getOrDefault(index > 0)
            roots.add(
                mapOf(
                    "path" to rootPath,
                    "label" to if (removable) "SD kart" else "Harici depolama",
                    "removable" to removable
                )
            )
        }
        invoke.resolveObject(mapOf("roots" to roots))
    }

    @Command
    fun storageInfo(invoke: Invoke) {
        val args = invoke.parseArgs(PathArgs::class.java)
        val path = args.path?.takeIf { it.startsWith("/") } ?: Environment.getExternalStorageDirectory().absolutePath
        try {
            val stat = StatFs(path)
            invoke.resolveObject(
                mapOf(
                    "total" to stat.blockCountLong * stat.blockSizeLong,
                    "free" to stat.availableBlocksLong * stat.blockSizeLong
                )
            )
        } catch (error: Exception) {
            fail(invoke, "Depolama bilgisi okunamadı: ${error.message}")
        }
    }

    @Command
    fun shareFiles(invoke: Invoke) {
        val args = invoke.parseArgs(PathsArgs::class.java)
        if (args.paths.isEmpty()) {
            fail(invoke, "Paylaşılacak dosya yok")
            return
        }
        try {
            val uris = args.paths.map { shareUri(it) }
            val intent = if (uris.size == 1) {
                Intent(Intent.ACTION_SEND).apply {
                    type = activity.contentResolver.getType(uris[0]) ?: "*/*"
                    putExtra(Intent.EXTRA_STREAM, uris[0])
                }
            } else {
                Intent(Intent.ACTION_SEND_MULTIPLE).apply {
                    type = "*/*"
                    putParcelableArrayListExtra(Intent.EXTRA_STREAM, ArrayList(uris))
                }
            }
            intent.addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
            intent.clipData = ClipData.newRawUri("Dosya Dolabı", uris[0]).apply {
                uris.drop(1).forEach { addItem(ClipData.Item(it)) }
            }
            activity.startActivity(Intent.createChooser(intent, "Paylaş"))
            invoke.resolveObject(mapOf("shared" to uris.size))
        } catch (error: Exception) {
            fail(invoke, "Paylaşım başlatılamadı: ${error.message}")
        }
    }

    @Command
    fun openWith(invoke: Invoke) {
        val args = invoke.parseArgs(OpenArgs::class.java)
        try {
            val uri = shareUri(args.path)
            val intent = Intent(Intent.ACTION_VIEW).apply {
                setDataAndType(uri, args.mime ?: activity.contentResolver.getType(uri) ?: "*/*")
                addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
            }
            activity.startActivity(Intent.createChooser(intent, "Birlikte aç"))
            invoke.resolveObject(mapOf("opened" to true))
        } catch (error: Exception) {
            fail(invoke, "Uygulama bulunamadı: ${error.message}")
        }
    }

    /** Kenardan kenara çizimde sistem çubuğu simgelerinin rengini eşitler. */
    @Command
    fun systemBars(invoke: Invoke) {
        val args = invoke.parseArgs(SystemBarsArgs::class.java)
        activity.runOnUiThread {
            val window = activity.window
            WindowCompat.setDecorFitsSystemWindows(window, false)
            val controller = WindowInsetsControllerCompat(window, window.decorView)
            controller.isAppearanceLightStatusBars = !args.dark
            controller.isAppearanceLightNavigationBars = !args.dark
        }
        invoke.resolveObject(mapOf("ok" to true))
    }

    /** Android 12+ duvar kağıdı paletinden vurgu rengi. */
    @Command
    fun dynamicAccent(invoke: Invoke) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            val color = activity.getColor(android.R.color.system_accent1_400)
            invoke.resolveObject(
                mapOf(
                    "color" to String.format("#%06X", 0xFFFFFF and color),
                    "available" to true
                )
            )
        } else {
            invoke.resolveObject(mapOf("color" to "", "available" to false))
        }
    }

    @Command
    fun takeIncoming(invoke: Invoke) {
        val files = synchronized(pendingIncoming) {
            val copy = pendingIncoming.toList()
            pendingIncoming.clear()
            copy
        }
        invoke.resolveObject(mapOf("files" to files))
    }

    @Command
    fun minimizeApp(invoke: Invoke) {
        activity.moveTaskToBack(true)
        invoke.resolveObject(mapOf("minimized" to true))
    }

    override fun onNewIntent(intent: Intent) {
        captureIntent(intent)
    }
}
