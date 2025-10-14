#!/usr/bin/env -S frida factorio -l
/*
b AchievementStats::allowed
p {bool}($rdi+0x21f)
p {bool}($rdi+0x220)
set {bool}($rdi+0x220)=0

00fc8398  80bb1f02000000     cmp     byte [rbx+0x21f {Map::disable_achievements_cheat}], 0x0
00fc839f  0f855affffff       jne     0xfc82ff
00fc83a5  80bb2002000000     cmp     byte [rbx+0x220 {Map::disable_achievements_editor}], 0x0
00fc83ac  0f854dffffff       jne     0xfc82ff
*/

DebugSymbol.findFunctionsMatching('*AchievementStats*allowed*').forEach(addr => {
    console.log('hooking function', addr);
    Interceptor.attach(addr, {
        onEnter: function (args) {
            // console.log('AchievementStats::allowed');
            let rbx = new NativePointer(this.context.rdi);
            if (!rbx.toString().startsWith('0x7')) {
                return;
            }
            let bools = [rbx.add(0x21f), rbx.add(0x220)];
            for (let x of bools) {
                if (x.readU8() != 0) {
                    console.log('Flag is non-zero:', x.readU8());
                    x.writeU8(0);
                    console.log('Resetting!');
                }
            }
        }
    });
});
