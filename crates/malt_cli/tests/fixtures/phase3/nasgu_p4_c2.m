x = 1;
try
    error('a:b', 'm');
catch x
end
disp(x)
