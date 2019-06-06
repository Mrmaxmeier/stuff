import gdb
from contextlib import contextmanager
import io

def ansi_aware_len(s):
	i = 0
	l = 0
	while "\033" in s[i:]:
		x = s.index("\033", i)
		l += x - i
		i = x
		i = s.index("m", i)+1
	return l + len(s[i:])


def clippy_str(lines):
	clippy = r"""
 __
/  \
|  |
@  @
|| ||
|| ||   <--
|\_/|
\___/
""".splitlines()
	clippy_width = max(map(len, clippy))
	box_height = len(lines) + 3
	box_width = max(map(ansi_aware_len, lines)) + 4
	height = max(box_height, len(clippy))
	clippy_start = height - len(clippy)
	box_start = max(len(clippy) - box_height, 0)
	s = ""
	for i in range(height):
		if i >= clippy_start:
			s += clippy[i-clippy_start].ljust(clippy_width)
		else:
			s += " " * clippy_width
		if i == box_start:
			s += " " + "_"*(box_width-2)
		elif i == box_start+1:
			s += "/" + " "*(box_width-2) + "\\"
		elif i == height-1:
			s += "\\" + "_"*(box_width-2) + "/"
		elif i >= box_start:
			line = lines[i-box_start-2]
			s += "| " + line + " "*(box_width-4-ansi_aware_len(line)) + " |"
		s += "\n"
	return s

# print(clippy_str(["It looks", "like you", "are writing", "a letter."]))
RESET  = "\033[0m"
GREEN  = "\033[0;32m"
YELLOW = "\033[0;33m"
PURPLE = "\033[0;35m"
BLUE   = "\033[0;36m"
WHITE  = "\033[0;37m"

@contextmanager
def clippy():
	s = io.StringIO()
	try:
		yield lambda *args, **kwargs: print(*args, **kwargs, file=s)
	finally:
		gdb.write(clippy_str(s.getvalue().split("\n")))

def fmt_get_location_from_symbol(address):
	# TODO: check stack frame symbols
	"""Retrieve the location of the `address` argument from the symbol table.
	Return a tuple with the name and offset if found, None otherwise."""
	# this is horrible, ugly hack and shitty perf...
	# find a *clean* way to get gdb.Location from an address
	name = None
	sym = gdb.execute("info symbol {:#x}".format(address), to_string=True)
	if sym.startswith("No symbol matches"):
		return None

	i = sym.find(" in section ")
	sym = sym[:i].split()
	name, offset = sym[0], 0
	if len(sym) == 3 and sym[2].isdigit():
		offset = int(sym[2])
	return name, offset



class FmtSegment:
	def __init__(self, position, flags, padding, length, conversion):
		self.position = position
		self.flags = flags
		self.padding = padding
		self.length = length
		self.conversion = conversion

	def __repr__(self):
		s = BLUE + "%"
		s += YELLOW + str(self.position) + BLUE + "$" # TODO: if position_explicit
		if self.flags: s += BLUE + self.flags
		if self.padding: s += YELLOW + str(self.padding)
		if self.length: s += BLUE + self.length
		s += BLUE + self.conversion + RESET
		return s

	def output_length(self, length_table):
		if self.conversion == "s":
			return None
		if self.conversion == "n":
			return 0
		if self.padding:
			return self.padding # might be wrong if output > padding
		return None # unknown size

	def typ(self, length_table):
		if self.length:
			typ = length_table[self.length]
		else:
			typ = length_table[self.conversion]
		if self.conversion == "n":
			typ = typ.pointer()
		return typ

flag_chars = "#0- +"
length_specifiers = ["ll", "hh", "h", "l", "q", "L", "j", "z", "Z", "t"]
conversion_specifiers = "diouxXeEfFgGaAcsCSpnm%"
def parse_format_string(fmtstr):
	position_counter = 1
	while "%" in fmtstr:
		i = fmtstr.index("%")
		pre = fmtstr[:i]
		if pre:
			yield pre
		fmtstr = fmtstr[i+1:]
		if not fmtstr:
			yield None
			break
		position = None
		if "$" in fmtstr:
			position = fmtstr[:fmtstr.index("$")]
			if str.isdigit(position) and position != "0":
				fmtstr = fmtstr[1+len(position):]
				position = int(position)
			else:
				position = None
		if position is None:
			position = position_counter
			position_counter += 1

		flags = ""
		while fmtstr and fmtstr[0] in flag_chars:
			flags += fmtstr[0]
			fmtstr = fmtstr[1:]
		if not fmtstr:
			yield None
			break
		flags = flags if flags else None

		padstr = ""
		while fmtstr and fmtstr[0] in "0123456789":
			padstr += fmtstr[0]
			fmtstr = fmtstr[1:]
		if not fmtstr:
			yield None
			break
		padding = int(padstr) if padstr else None

		length_specifier = None
		for ls in length_specifiers:
			if fmtstr.startswith(ls):
				length_specifier = ls
				fmtstr = fmtstr[len(ls):]
				break
		if not fmtstr or fmtstr[0] not in conversion_specifiers:
			yield None
			break
		cs = fmtstr[0]
		fmtstr = fmtstr[1:]

		if length_specifier:
			if length_specifier in "hhllqjzZt" and cs not in "diouxX" + "n":
				yield None
				break
			if length_specifier == "L" and cs not in "aAeEfFgG":
				yield None
				break

		yield FmtSegment(position, flags, padding, length_specifier, cs)
	if fmtstr:
		yield fmtstr

tests = """
%s %x
%123$hhx
%123$s %123$hhn
%%
%65c%1$n
%1$0#16hx
%42c%11$n
%11$n%256c%42c%11$hhn
""".strip()
for test in tests.splitlines():
	assert all(parse_format_string(test)), test # no parse errors


class FmtDbg:
	def __init__(self):
		self.update_arch()
		self.update_callsite()
		self.update_fmtstr()

	def update_arch(self):
		frame = gdb.selected_frame()
		arch = frame.architecture().name()
		self.arch = arch
		assert arch in ["i386", "i386:x86-64"]

		self.length_modifiers = {
			"hh": gdb.lookup_type("unsigned char"),
			"h":  gdb.lookup_type("unsigned short"),
			"l":  gdb.lookup_type("unsigned long"),
			"ll": gdb.lookup_type("unsigned long long"),
			"q":  gdb.lookup_type("unsigned long long"),
			"L":  gdb.lookup_type("long double"),
			"p":  gdb.lookup_type("void").pointer(),
			"d":  gdb.lookup_type("unsigned int"),
			"i":  gdb.lookup_type("unsigned int"),
			"o":  gdb.lookup_type("unsigned int"),
			"u":  gdb.lookup_type("unsigned int"),
			"x":  gdb.lookup_type("unsigned int"),
			"X":  gdb.lookup_type("unsigned int"),
			"c":  gdb.lookup_type("unsigned char"),
			"n":  gdb.lookup_type("unsigned int"),
			"s":  gdb.lookup_type("unsigned char").pointer(),
		}
		# TODO: incomplete

	def update_callsite(self):
		func  = gdb.selected_frame().name()
		family_parameter_offset = {
			"printf": 0,
			"fprintf": 1,
			"dprintf": 1,
			"sprintf": 1,
			"snprintf": 2,
		}
		if func not in family_parameter_offset:
			print("[!] unknown format string function. assuming printf function signature")
		self.format_string_position = family_parameter_offset.get(func, 0)

	def update_fmtstr(self):
		frame = gdb.selected_frame()
		arch = frame.architecture().name()
		assert arch in ["i386", "i386:x86-64"]
		char_ptr_type = gdb.lookup_type("char").pointer()
		charptr = self.fmtstr_arg(0, char_ptr_type)[1]
		self.fmtstr = charptr.string("latin1")

	def fmtstr_arg(self, idx, typ, pad=0):
		idx += self.format_string_position
		if self.arch == "i386":
			v = f"$sp+{(idx+1)*4}"
			s = f"$sp[{idx+1:0{pad}}]"
			return (s, gdb.parse_and_eval(v).cast(typ.pointer()).dereference())
		elif self.arch == "i386:x86-64":
			if idx < 6:
				reg = ["$rdi", "$rsi", "$rdx", "$rcx", "$r8", "$r9"][idx]
				return (reg, gdb.parse_and_eval(reg).cast(typ))
			v = f"$sp+{(idx-6+1)*8}"
			s = f"$sp[{idx-5:0{pad}}]"
			return (s, gdb.parse_and_eval(v).cast(typ.pointer()).dereference())
		else:
			assert False, "unsupported architecture"

	def context(self):
		with clippy() as clp:
			clp("It looks like you're trying to write a format string:")
			clp(GREEN + repr(self.fmtstr) + RESET)
			segments = list(parse_format_string(self.fmtstr))
			if any([x is None for x in segments]):
				clp("I don't understand this format string.")
				return
			clp()
			clp("This format string consists of:")
			outcount = 0
			for segment in segments:
				clp("-", GREEN + repr(segment) + RESET)
				if isinstance(segment, str):
					if outcount is not None:
						outcount += len(segment)
					continue

				assert isinstance(segment, FmtSegment)
				arg_typ = segment.typ(self.length_modifiers)
				arg_s, arg_v = self.fmtstr_arg(segment.position, arg_typ)
				action = ""
				if segment.conversion == "n":
					action += f"write "
					if outcount is not None:
						writewidth = arg_typ.target().sizeof
						mask = (1<<writewidth*8)-1
						action += f"{YELLOW}{outcount & mask:#0{writewidth*2+2}x}{RESET}"
					else:
						action += f"{YELLOW}???{RESET}"
					action += f" to address at "
				else:
					action += "leak "
				action += f"{BLUE}{arg_s}{RESET}"
				if arg_v.address:
					action += f" (*{hex(arg_v.address)})"
				action += f": {YELLOW}{hex(arg_v)}{RESET}"
				if fmt_get_location_from_symbol(int(arg_v)):
					symb, offset = fmt_get_location_from_symbol(int(arg_v))
					action += f" {PURPLE}<&{symb}"
					if offset:
						action += f"+{offset}"
					action += f">{RESET}"
				clp(action)
				seglen = segment.output_length(self.length_modifiers)
				if seglen is not None and outcount is not None:
					outcount += seglen
				else:
					outcount = None


class FmtDbgHook(gdb.Command):
	def __init__(self):
		super(FmtDbgHook, self).__init__("fmtdbg-hook", gdb.COMMAND_USER)

	def invoke(self, arg, from_tty):
		targets = set(gdb.string_to_argv(arg) if arg else ["printf"])
		covered = [x.location for x in gdb.breakpoints() if isinstance(x, PrintfBreakpoint)]
		for target in targets:
			if target in covered:
				print(target, "already hooked")
			else:
				print("hooking", target)
				PrintfBreakpoint(target)

class PrintfBreakpoint(gdb.Breakpoint):
	def stop (self):
		FmtDbg().context()
		return True

class FmtDbgStack(gdb.Command):
	def __init__(self):
		super(FmtDbgStack, self).__init__("fmtdbg-stack", gdb.COMMAND_USER)
		self.repeat_count = 0
		self.repeat = False
		self.__last_command = None

	def __repeat_count_hack(self, arg, from_tty):
		if not from_tty:
			self.repeat_count = 0
			self.repeat = False
			self.__last_command = None
			return
		command = gdb.execute("show commands", to_string=True).strip().split("\n")[-1]
		self.repeat = self.__last_command == command
		self.repeat_count = self.repeat_count + 1 if self.repeat else 0
		self.__last_command = command

	def invoke(self, arg, from_tty):
		self.__repeat_count_hack(arg, from_tty)
		args = gdb.string_to_argv(arg)
		size = 16
		if args:
			size = int(args[0])
		typ = gdb.lookup_type("void").pointer()
		#with clippy() as clp:
		fmtdbg = FmtDbg()
		idx = 1 + size*self.repeat_count
		for i in range(idx, idx+size):
			arg_s, arg_v = fmtdbg.fmtstr_arg(i, typ, pad=3)
			line = f"{GREEN}\"%{i:03}$p\"{RESET}: "
			line += f"{BLUE}{arg_s}{RESET}"
			if arg_v.address:
				line += f" (*{hex(arg_v.address)})"
			line += f": {YELLOW}{int(arg_v):#0{typ.sizeof*2+2}x}{RESET}"
			if fmt_get_location_from_symbol(int(arg_v)):
				symb, offset = fmt_get_location_from_symbol(int(arg_v))
				line += f" {PURPLE}<&{symb}"
				if offset:
					line += f"+{offset}"
				line += f">{RESET}"
			print(line)
			#clp(line)

fmtdbghook = FmtDbgHook()
fmtdbgstack = FmtDbgStack()
