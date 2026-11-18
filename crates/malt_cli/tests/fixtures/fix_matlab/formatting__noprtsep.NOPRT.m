function out = noprtsep(u, v)
%NOPRTSEP Comma-separated statements in a function get semicolons.
p = u + 1; r = v + 2;
s = 3;
if p > 1, out = s; else, out = r; end
t = 4; % c
end
