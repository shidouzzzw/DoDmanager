# SPDX-License-Identifier: GPL-3.0-or-later
# Rust tarafı Kotlin eklentisini sınıf adıyla yükler; R8 bu adları korumalı.
-keep class com.dod.manager.AndroidFsPlugin { *; }
-keep class com.dod.manager.MainActivity { *; }

# Tauri köprüsü ve @Command / @ActivityCallback metotları yansımayla çağrılır.
-keep class app.tauri.** { *; }
-keepclassmembers class * {
    @app.tauri.annotation.Command <methods>;
    @app.tauri.annotation.ActivityCallback <methods>;
    @app.tauri.annotation.PermissionCallback <methods>;
}

-dontwarn app.tauri.**
