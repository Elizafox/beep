# Makefile for regenerating the beep(1) man page.

.PHONY: all man check clean

all: man

man: beep.1

beep.1: beep.1.md
	pandoc -s -t man $< -o $@

check: beep.1.md
	pandoc -s -t man beep.1.md -o beep.1.generated
	@if diff -q beep.1 beep.1.generated >/dev/null; then \
		echo "beep.1 is up to date."; \
		rm -f beep.1.generated; \
	else \
		echo "beep.1 is out of date. Run 'make man' to regenerate."; \
		diff beep.1 beep.1.generated; \
		rm -f beep.1.generated; \
		exit 1; \
	fi

clean:
	rm -f beep.1.generated
