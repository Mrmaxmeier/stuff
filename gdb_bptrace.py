# TODO: this kind of depends on GEF

from collections import Counter, defaultdict

import gdb

__trace__ = None


class TraceLog:
    def __init__(self, enabled=None):
        self.events = []
        self.enabled = enabled
        if self.enabled is None:
            self.enabled = [b.number for b in gdb.breakpoints()]
        self.ranges = [(bid, 0, None) for bid in self.enabled]
        self.bp_hits = defaultdict(int)
    
    def prepare(self):
        print("[*] resetting hit counts")
        for bp in gdb.breakpoints():
            bp.hit_count = 0
            bp.silent = True
            bp.enabled = bp.number in self.enabled
        print("[*] enabled breakpoints: ", self.enabled)
    
    def update(self):
        for bp in gdb.breakpoints():
            if bp.hit_count != self.bp_hits[bp.number]:
                self.bp_hits[bp.number] = bp.hit_count
                print(f"[{len(self.events)}] hit bp {bp.number} ({bp.location})")
                self.events.append(bp.number)

    def seek(self, index):
        # TODO: prefer path of least stops
        bpnum = self.events[index]
        was_enabled = set()
        for bp in gdb.breakpoints():
            if bp.enabled:
                was_enabled.add(bp.number)
            bp.enabled = bp.number == bpnum
        skipcount = self.events[:index].count(bpnum)
        print("[*] seeking...")
        gdb.execute("run")
        for i in range(skipcount):
            gdb.execute("continue")
            # print(f"\u001b[1000D[*] {i+1} / {skipcount}")
        for bp in gdb.breakpoints():
            bp.enabled = bp.number in was_enabled
    
    def print(self, event_filter=None):
        s = []
        loc = {bp.number: bp.location for bp in gdb.breakpoints()}

        if event_filter:
            for i, n in enumerate(self.events):
                context = self.events[max(0, i-2):min(len(self.events), i+2)]
                if any(map(event_filter, context)):
                    s.append(f"[{i}] {n} {loc[n]}")
        else:
            s = [f"[{i}] {n} {loc[n]}" for i, n in enumerate(self.events)]
        gdb.write("\n".join(s) + "\n")



class SilencedGEF:
    def __init__(self):
        self.paused_hooks = []
        try:
            self.to_pause = [ContextCommand.update_registers, ida_synchronize_handler]
        except Exception as e:
            print(e)
            self.to_pause = []
        # TODO: pause stop handler

    def __enter__(self):
        gdb.execute("gef config context.enable False")
        for hook in self.to_pause:
            try:
                gdb.events.cont.disconnect(hook)
                self.paused_hooks.append(hook)
            except SystemError:
                pass

    def __exit__(self, *args):
        for hook in self.paused_hooks:
            gdb.events.cont.connect(hook)
        gdb.execute("gef config context.enable True")


class TraceBreakpoints(gdb.Command):
    """Assign an ID to every breakpoint event. This needs deterministic runs without stdin reads."""

    def __init__(self):
        super(TraceBreakpoints, self).__init__("trace-breakpoints", gdb.COMMAND_USER)

    def invoke(self, arg, from_tty):
        if arg.startswith("seek "):
            n = int(arg[len("seek "):])
            with SilencedGEF():
                __trace__.seek(n)
            gdb.execute("ctx")
        elif arg.startswith("refine "):
            self.refine(0, (0, 0))
        elif arg.startswith("log "):
            args = arg[len("log "):].split(" ")
            loc = {bp.location: bp.number for bp in gdb.breakpoints()}
            ids = [loc[i] if i in loc else int(i) for i in args]
            __trace__.print(event_filter=lambda e: e in ids)
        elif arg.startswith("log"):
            __trace__.print()
        else:
            self.new_trace([int(n) for n in arg.split(" ")] if arg != "" else None)

    def new_trace(self, enabled_breakpoints):
        global __trace__
        __trace__ = TraceLog(enabled=enabled_breakpoints)
        __trace__.prepare()
        with SilencedGEF():
            gdb.execute("run")
            __trace__.update()
            try:
                while len(__trace__.events) < 50_000:
                    gdb.execute("continue")
                    __trace__.update()
                else:
                    print("[*] reached 50000 hits, try refining the breakpoint selection")
            except gdb.error as e:
                print(e)
            except KeyboardInterrupt:
                print("[*] sigint")
            print(Counter(__trace__.events))

    def refine(self, bp, id_range):
        pass
TraceBreakpoints()
