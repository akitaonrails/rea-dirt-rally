# Third-party components and assets

Nothing from DiRT Rally itself is part of this repository or its packages.
Every component below is fetched from a pinned URL and checked by SHA-256
(`packaging/third-party.txt`), or is an asset with its licence recorded.

| Component | Version | What it does here | Licence | Source | Bundled? |
|---|---|---|---|---|---|
| [rea-launcher-sdk](https://github.com/akitaonrails/rea-launcher-sdk) | v0.3.3 | Shared launcher code, Linux helper, packaging | MIT OR Apache-2.0 | https://github.com/akitaonrails/rea-launcher-sdk | compiled in, `sdk/` |
| Barlow fonts | from the SDK | The settings window's type | SIL OFL 1.1 (`licenses/Barlow-OFL.txt`) | https://github.com/jpt/barlow | compiled in |
| quick-xml | 0.41 | Reads the game's `preferences` file | MIT | https://github.com/tafia/quick-xml | compiled in |
| libc (crate) | 0.2 | System calls in the exec-read library | MIT OR Apache-2.0 | https://github.com/rust-lang/libc | compiled in |
| Legacy libraries for the Linux port (below) | Steam Runtime 1 "scout" | Libraries DiRT Rally's 2017 Linux build needs that modern systems dropped | per library (below); each package's `copyright` ships in `licenses/compat/` | https://repo.steampowered.com/steamrt/ | `game-libs/compat/` |

## Legacy libraries (`game-libs/compat`)

Taken at package time from the Steam Runtime 1 ("scout") packages pinned
in `packaging/third-party.txt`; only the sonames in
`packaging/compat-libs.txt` ship, and only to the game's process (never to
the launcher's own). The corresponding sources are the source packages of
the same versions in the same repository
(`https://repo.steampowered.com/steamrt/pool/main/<letter>/<source>/`).

| Library (soname) | Package | Licence |
|---|---|---|
| libssl.so.1.0.0, libcrypto.so.1.0.0 | openssl 1.0.1 | OpenSSL / SSLeay |
| libidn.so.11 | libidn 1.32 | LGPL-2.1-or-later (library) |
| librtmp.so.0 | rtmpdump 2.4 | LGPL-2.1 |
| libldap_r-2.4.so.2, liblber-2.4.so.2 | openldap 2.4.28 | OpenLDAP Public License 2.8 |
| libgconf-2.so.4 | gconf 3.2.5 | LGPL-2.0-or-later |
| libgnutls.so.26 | gnutls26 2.12.14 | LGPL-2.1-or-later (library) |
| libgcrypt.so.11 | libgcrypt11 1.5.0 | LGPL-2.1-or-later |
| libasn1, libgssapi, libhcrypto, libheimbase, libheimntlm, libhx509, libkrb5, libroken, libwind | heimdal 1.6 | BSD-3-Clause |
| libnettle.so.6, libhogweed.so.4 | nettle 3.2 | LGPL-3.0-or-later / GPL-2.0-or-later (dual) |
| libdbus-glib-1.so.2 | dbus-glib 0.98 | AFL-2.1 OR GPL-2.0-or-later |
| libffi.so.6 | libffi 3.0.11 | MIT |
| libsasl2.so.2 | cyrus-sasl2 2.1.25 | BSD-style (CMU) |
| libtasn1.so.3 | libtasn1-3 2.10 | LGPL-2.1-or-later |

## Our own code that touches the game

| File | What | Licence |
|---|---|---|
| `crates/dirtrally-exec-read` | LD_PRELOAD library: `PROT_EXEC` mappings also get `PROT_READ` (PKU crash). Written for this launcher; the community fix it follows (an `mprotect` shim posted in FeralInteractive/ferallinuxscripts#3) has no licence and is not used. | MIT OR Apache-2.0 |

## Images

| File | Author | Licence | Source |
|---|---|---|---|
| (the window's hero image, once researched) | | | |

## Owner decisions

- **Pre-activated copies (2026-11-02).** DiRT Rally is no longer sold
  (withdrawn on 3 Oct 2022) and its online services ended on 8 Nov 2025.
  The owner opened an exception to the SDK's DRM rule for this game: the
  launcher accepts a copy that already runs without Steam, identified by
  SHA-256, and runs it unchanged. The launcher never applies a crack and
  never adds, replaces or patches Steam or copy-protection files; the
  README says a pre-activated copy is needed and that we can't and won't
  explain how to get one.
