// SPDX-License-Identifier: GPL-3.0-or-later
package com.dod.manager

import android.os.Bundle
import android.webkit.ValueCallback
import android.webkit.WebView
import androidx.activity.OnBackPressedCallback
import androidx.core.view.WindowCompat

/**
 * Android geri tuşu/hareketi: önce arayüzdeki `window.__dodBack()` denenir.
 * Arayüz `true` dönerse (sheet kapandı, klasörden çıkıldı) varsayılan davranış
 * engellenir; aksi halde etkinlik kapanır.
 */
class MainActivity : TauriActivity() {
    private var webView: WebView? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // Kenardan kenara çizim; güvenli alan CSS env() ile yönetilir.
        WindowCompat.setDecorFitsSystemWindows(window, false)

        onBackPressedDispatcher.addCallback(
            this,
            object : OnBackPressedCallback(true) {
                override fun handleOnBackPressed() {
                    val view = webView
                    if (view == null) {
                        goBack()
                        return
                    }
                    view.evaluateJavascript(
                        "(function(){try{return window.__dodBack ? window.__dodBack() === true : false}catch(e){return false}})()",
                    ) { result: String? ->
                        if (result != "true") goBack()
                    }
                }
            },
        )
    }

    private fun goBack() {
        onBackPressedDispatcher.onBackPressed()
    }

    override fun onWebViewCreate(webView: WebView) {
        this.webView = webView
        // Geri çağrısı için JS köprüsü hazır olduğundan emin olunur.
        webView.evaluateJavascript("window.__dodBack = window.__dodBack || null", null as ValueCallback<String>?)
    }
}
