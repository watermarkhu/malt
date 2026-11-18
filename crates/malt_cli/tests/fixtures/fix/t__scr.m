for scr = 1:3, end
[scr, q] = deal(1,2);
x = struct(); x.scr = 1;
scr(2) = 5;
global scr
