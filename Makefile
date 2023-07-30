program.bin : program.ch8
	@python ../Chip8Assembler/assembler.py --output program.bin ./program.ch8
