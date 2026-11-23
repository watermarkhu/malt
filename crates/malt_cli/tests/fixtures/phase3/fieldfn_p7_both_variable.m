s = struct('a', 1);
s = setfield(s, 'a', 3);
u = getfield(s, 'a');
disp(u)
