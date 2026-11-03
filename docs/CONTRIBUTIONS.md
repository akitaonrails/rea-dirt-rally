# Contributions back

Fixes found while building this launcher that the community lacks, drafted
for PCGamingWiki, upstream projects, ProtonDB or the WineHQ AppDB
(rea-launcher-sdk `docs/playbooks/contribute-back.md`). These are public
posts under the owner's name: agents only draft them here; the owner
reviews and posts. Only fixes verified in the real game.

| # | Target (page or project) | Fix | Evidence | Status |
|---|---|---|---|---|
| 1 | PCGamingWiki, [DiRT Rally](https://www.pcgamingwiki.com/wiki/DiRT_Rally), Issues fixed (Linux) | Crash right after start on CPUs with memory protection keys (PKU): execute-only memory made readable by an `LD_PRELOAD` library | RESEARCH.md #5; RUNTIME_NOTES.md "first headless runs", "native port plays a stage", "launcher pieces against the real game" | draft |
| 2 | PCGamingWiki, [DiRT Rally](https://www.pcgamingwiki.com/wiki/DiRT_Rally), Input (Linux) | Newer gamepads seen only as a joystick: add them to Feral's `share/inputdevices.json` | RUNTIME_NOTES.md "owner's session" (SDL call log, the owner's check across a restart) | draft |

## Drafts

Before posting either draft, the owner checks that the page (and its talk
page) does not already have the fix, and reads the wikitext once more
against the page's current fix-box style. Neither draft links or attaches
any game file.

### 1. Linux: crash after "SDL2 initialised" on Ryzen 5000+ and other PKU CPUs

**Applies to:** Feral Interactive's Linux port 1.0RC14 (Steam BuildID 31946,
branch `DirtRally_1_0`, dated 2017-03-01), `bin/DirtRally` SHA-256
`de81c75ce02318d6ba89d3d24c463ac311a823d9eb16e676e7a5fb66d6c9e39e`.

**Verified on:** AMD Ryzen 9 7950X3D (PKU), Linux 7.2, Arch Linux; the game
reached its menus and drove a full `-benchmark` stage, and the owner played
it, with the library preloaded. Without it, the crash below every time.

Reviewer note (not part of the paste): the digest is the owner's copy's
`bin/DirtRally`; no other copy was compared. If in doubt, cite only the
version and BuildID, which the game's own files state.

Wikitext, ready to paste under "Issues fixed":

```
===Crash on startup on AMD Ryzen 5000 and newer CPUs (Linux)===
{{Fixbox|description=Make execute-only memory readable|ref=<ref>{{Refcheck|user=<owner's wiki user>|date=2026-11-02|comment=Linux port 1.0RC14 (BuildID 31946), Ryzen 9 7950X3D, Linux 7.2.}}</ref>|fix=
The Linux version crashes (segmentation fault) on the main thread right after its log shows <code>SDL2 initialised</code>, before any window appears. The game maps memory with execute permission only (<code>PROT_EXEC</code> without <code>PROT_READ</code>) and then reads it. On CPUs with Memory Protection Keys (<code>pku</code> in <code>/proc/cpuinfo</code>: AMD Zen 3 and newer, Intel Xeon Scalable and some recent Intel CPUs), Linux 4.6 and later make such memory really execute-only, so the read faults. On older CPUs execute permission implied read permission, which is why the game worked when it was released.

# Check that your CPU has the feature: <code>grep -wo pku /proc/cpuinfo</code> prints <code>pku</code>.
# Preload a small library that adds <code>PROT_READ</code> to every <code>mmap</code>, <code>mmap64</code> and <code>mprotect</code> call that asks for <code>PROT_EXEC</code>, and changes nothing else. The [https://github.com/akitaonrails/rea-dirt-rally rea-dirt-rally] launcher builds one (<code>dirtrally-exec-read</code>, MIT or Apache-2.0) and preloads it for you.
# If you start the game yourself, add the library after the game's own Steam API library, by name, with its folder in <code>LD_LIBRARY_PATH</code> (a path with a space would split <code>LD_PRELOAD</code>):<br><code>LD_PRELOAD="libsteam_api.so:libdirtrally_exec_read.so"</code>

'''Notes'''
{{ii}} Capping the CPU count the game sees (<code>get_nprocs</code>/<code>sysconf</code>) does not fix this crash.
{{ii}} On current distributions the game also needs <code>lib/x86_64</code> in <code>LD_LIBRARY_PATH</code> (glibc 2.37 and later no longer expand <code>$PLATFORM</code> in its RPATH) and the old libraries it was built against (OpenSSL 1.0.0, libidn11 and others); see the other Linux issues on this page.
}}
```

(`<owner's wiki user>`: the owner fills in their wiki user name.)

### 2. Linux: newer gamepads only work as a joystick, or not at all

**Applies to:** the same Linux port 1.0RC14 (BuildID 31946). The game keeps
its own controller list in `share/inputdevices.json`.

**Verified on:** an 8BitDo Ultimate 2 Wireless Controller (USB ID
`2dc8:310b`, Linux `xpad` driver). Before: the game saw a plain joystick,
started on the keyboard, and a hand-made mapping was gone after a restart.
With the entry below: the Controls menu offers "Controller preset 1", which
came mapped correctly, and the game started on the gamepad with that preset
after a restart (owner's check). An SDL call log showed the game adding the
entry's mapping and opening the pad as a game controller.

Wikitext, ready to paste under "Input" (or "Issues fixed"):

```
===Newer gamepads are not recognised (Linux)===
{{Fixbox|description=Add the gamepad to the game's own controller list|ref=<ref>{{Refcheck|user=<owner's wiki user>|date=2026-11-02|comment=Linux port 1.0RC14 (BuildID 31946), 8BitDo Ultimate 2 Wireless (2dc8:310b, xpad).}}</ref>|fix=
The Linux version uses its own copy of SDL 2.0.5 (2016) and its own list of known controllers, <code>share/inputdevices.json</code> in the game folder. A pad that is not in that list is treated as a plain joystick: the game starts on the keyboard, offers no gamepad preset, and a mapping made by hand is lost on the next start. Setting <code>SDL_GAMECONTROLLERCONFIG</code> makes it worse: the game then drops the pad entirely.

# Find the pad's vendor and product IDs: <code>lsusb</code>, or <code>cat /sys/class/input/js0/device/id/vendor /sys/class/input/js0/device/id/product</code>.
# Back up <code>share/inputdevices.json</code>, then add an entry for the pad before the closing <code>]</code> (add a comma after the previous entry). Example for an 8BitDo Ultimate 2 Wireless on the <code>xpad</code> driver:
<pre>
{
    "Name": "8BitDo Ultimate 2 Wireless Controller",
    "VendorID": "0x2dc8",
    "ProductIDs": ["0x310b"],
    "Category": "Gamepad",
    "Type": "Xbox360Gamepad",
    "SDLMapping": "a:b0,b:b1,x:b2,y:b3,leftshoulder:b4,rightshoulder:b5,back:b6,start:b7,guide:b8,leftstick:b9,rightstick:b10,leftx:a0,lefty:a1,lefttrigger:a2,rightx:a3,righty:a4,righttrigger:a5,dpup:h0.1,dpright:h0.2,dpdown:h0.4,dpleft:h0.8"
}
</pre>
# Start the game, open Options → Controls → Controls preset and choose '''Controller preset 1'''.

'''Notes'''
{{ii}} The mapping above fits pads whose Linux driver reports the standard gamepad layout (most Xbox-style pads on <code>xpad</code> or <code>hid-generic</code>). Other pads need their own button and axis numbers.
{{ii}} A Steam update or "verify files" may put the original list back. The [https://github.com/akitaonrails/rea-dirt-rally rea-dirt-rally] launcher instead adds every connected standard-layout pad to a list in its own copy of the game at each start, and leaves the game folder unchanged.
}}
```
