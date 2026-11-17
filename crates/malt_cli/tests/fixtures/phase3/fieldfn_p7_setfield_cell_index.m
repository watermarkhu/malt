c = {struct('a', 1)};
c{1} = setfield(c{1}, 'a', 1);
disp(c)
