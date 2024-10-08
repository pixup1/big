# Python script used to create term_colors.rs from the hex in term_colors.txt
# No need to re-run unless the colors are incorrect for some reason

f = open("term_colors.txt", "r")
nf = open("src/term_colors.rs", "w")
nf.write("// Generated by ../parse_colors.py\n\n")
nf.write("use crate::style::Color;\n\n")
nf.write("pub const TERM_COLORS: [Color; 256] = [\n")
lines = f.readlines()
for line in lines:
	nf.write("    Color::new_rgb(")
	nf.write(str(int(line[0:1], 16)) + ", ")
	nf.write(str(int(line[2:3], 16)) + ", ")
	nf.write(str(int(line[4:5], 16)) + "")
	nf.write("),\n")
f.close()
nf.write("];")
nf.close()