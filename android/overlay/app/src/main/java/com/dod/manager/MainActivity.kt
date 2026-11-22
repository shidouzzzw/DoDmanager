// SPDX-License-Identifier: GPL-3.0-or-later
package com.dod.manager

import android.os.Bundle
import android.webkit.WebView
import androidx.activity.OnBackPressedCallback
import androidx.core.view.WindowCompat

/**
 * Android geri tuşu/hareketi: önce arayüzdeki `window.__dodBack()` denenir.
 * Arayüz `true` dönerse (sheet kapandı, klasörden çıkıldı, seçim temizlendi)
 * varsayılan davranış engellenir; aksi halde etkinlik kapanır.
 */
class MainActivity : TauriActivity() {
    private var webView: WebView? = null

    private val backCallback = object : OnBackPressedCallback(false) {
        override fun handleOnBackPressed() {
            val view = webView
            if (view == null) {
                finishActivity()
                return
            }
            view.evaluateJavascript(JS_BACK) { result ->
                if (result != "true") finishActivity()
            }
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // Kenardan kenara çizim; güvenli alanlar CSS env() ile yönetilir.
        WindowCompat.setDecorFitsSystemWindows(window, false)
        onBackPressedDispatcher.addCallback(this, backCallback)
        backCallback.isEnabled = true
    }

    /** Geri çağrısını devre dışı bırakıp varsayılan davranışı (çıkış) çalıştırır. */
    private fun finishActivity() {
        backCallback.isEnabled = false
        onBackPressedDispatcher.onBackPressed()
    }

    override fun onWebViewCreate(webView: WebView) {
        this.webView = webView
    }

    private companion object {
        const val JS_BACK =
            "(function(){try{return window.__dodBack ? window.__dodBack() === true : false}catch(e){return false}})()"
    }
}
