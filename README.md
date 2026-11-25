<div align="center">

# ⚡ StreamBoost

**Windows 10/11 için Otomatik Oyun Performans Yöneticisi / Automatic Game Performance Manager for Windows 10/11**

[![Platform](https://img.shields.io/badge/Platform-Windows%2010%2F11-06b6d4?style=flat-square&logo=windows)](https://github.com/Jung1330/StreamBoost)
[![Tech](https://img.shields.io/badge/Built%20With-Rust%20%2B%20Tauri%20v2%20%2B%20React-10b981?style=flat-square)](https://github.com/Jung1330/StreamBoost)
[![AI Assisted](https://img.shields.io/badge/AI--Assisted-Vibe%20Coding-8b5cf6?style=flat-square)](https://github.com/Jung1330/StreamBoost)
[![License](https://img.shields.io/badge/License-MIT-slate?style=flat-square)](LICENSE)

> **🤖 Şeffaflık Notu / Transparency Note:**  
> *Bu proje yapay zekâ (AI) desteğiyle geliştirilmiştir. / Built with AI assistance.*

</div>

---

## 🇹🇷 Türkçe

**StreamBoost**, Windows 10/11 üzerinde oyun açıldığında otomatik olarak devreye girip sistem kaynaklarını oyuna odaklayan, oyun kapandığında ise **tüm ayarları eksiksiz eski haline döndüren** açık kaynaklı, hafif ve modern bir performans yöneticisidir.

### ✨ Temel Özellikler
- **Otomatik Oyun Algılama & Profil Sistemi:** Açık uygulamalardan tek tıkla veya manuel olarak `.exe` ekleyin. Düşük maliyetli izleyici (2.5 sn aralık, `<%0.5 CPU`) oyunu algılayınca **Boost Aktif** moduna geçer; oyun veya StreamBoost kapatıldığında her şeyi geri alır.
- **CPU Önceliği & Çekirdek Ataması (Affinity):** Aktif oyuna güvenli biçimde `HIGH_PRIORITY_CLASS` uygular *(sistem kilitleme riskine karşı asla `REALTIME` kullanılmaz)*. Oyun bazlı mantıksal CPU çekirdeği seçimi sunar.
- **Arka Plan Yükünü Kısma (Windows EcoQoS):** Oyun açıkken tarayıcı, Discord, Spotify, launcher gibi seçili arka plan uygulamalarını resmi Windows `SetProcessInformation(ProcessPowerThrottling)` API'si ile **Verimlilik Moduna (EcoQoS)** iter; oyun kapanınca normale döndürür.
- **Güç Planı & 1ms Zamanlayıcı:** Oyun açılınca mevcut Windows güç planını kaydedip **Yüksek Performans** planına (`powercfg`) geçer; opsiyonel olarak `timeBeginPeriod(1)` uygular. Kapanınca eski güç planını ve `timeEndPeriod(1)` ayarını otomatik geri yükler.
- **Canlı Durum & Sistem Tepsisi (System Tray):** Canlı CPU/RAM göstergeleri, aktif optimizasyon rozetleri, tek tıkla manuel Boost aç/kapa ve anlık **TR / EN** dil değiştirici. Pencere kapatılınca sistem tepsisinde sessiz çalışır.

### 🛡️ Güvenlik Kırmızı Çizgileri
- Kritik Windows sistem süreçleri (`System`, `csrss.exe`, `lsass.exe`, `dwm.exe`, `svchost.exe`, `explorer.exe` vb.) ve anti-cheat servisleri **asla değiştirilemez**.
- Oyun belleğine **asla inject / memory erişimi yapılmaz** — yalnızca standart Windows süreç ve güç API'leri kullanılır.

---

## 🇬🇧 English

**StreamBoost** is an open-source, lightweight, modern Windows 10/11 performance manager that automatically optimizes CPU priority, core affinity, background process power throttling (EcoQoS), power plans, and timer resolution when a tracked game launches—and cleanly restores every setting when the game exits.

### ✨ Key Features
- **Automatic Game Detection & Profiles:** Add games from running apps in one click or manually by `.exe`. A low-overhead watcher (2.5s interval, `<0.5% CPU`) activates **Boost Mode** automatically and restores everything when the game or StreamBoost exits.
- **CPU Priority & Core Affinity:** Safely elevates the active game to `HIGH_PRIORITY_CLASS` *(never uses `REALTIME`)* with optional per-game logical CPU core affinity selection.
- **Background Throttling (Windows EcoQoS):** Places selected background apps (browsers, Discord, Spotify, game launchers) into Windows **Efficiency Mode** via the official `SetProcessInformation(ProcessPowerThrottling)` API while gaming, and restores them afterward.
- **Power Plan & 1ms System Timer:** Saves the active Windows power plan, switches to **High Performance** (`powercfg`), and optionally enables `timeBeginPeriod(1)` during gameplay—restoring the original plan and calling `timeEndPeriod(1)` on exit.
- **Live Dashboard & System Tray:** Real-time CPU/RAM telemetry, optimization badges, one-click Manual Boost toggle, instant **TR / EN** language switcher, and silent System Tray operation.

### 🛡️ Safety Guarantees
- Critical Windows system processes (`System`, `csrss.exe`, `lsass.exe`, `dwm.exe`, `svchost.exe`, `explorer.exe`, etc.) and anti-cheat services are strictly protected and never modified.
- **Zero memory access or code injection** — uses only standard, documented Win32 Process, Power, and Multimedia Timer APIs.

---

## 🛠️ Geliştirme / Development

```bash
# Bağımlılıkları yükle / Install dependencies
npm install

# Geliştirme modunda çalıştır / Run in dev mode
npm run tauri dev

# Release derlemesi al / Build release binary
npm run tauri build
```

---

<div align="center">
<sub>Bu proje yapay zekâ (AI) desteğiyle geliştirilmiştir. / Built with AI assistance.</sub>
</div>
