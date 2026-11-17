function out = nbrak2fix(a, c)
%NBRAK2FIX Bracket stripping keeps the inner text, comments and all.
y = a(1) ;
z =  -1 ;
m = a  % keep this comment
;
n =  a ;
out = [y, z, m, n, c];
end
