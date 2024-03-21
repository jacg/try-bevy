# -*-Makefile-*-

test colours='':
     cargo {{colours}} nextest run

run:
     cargo -q run
