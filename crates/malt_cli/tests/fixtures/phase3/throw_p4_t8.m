function throw_p4_t8()
x = 1;
try
    throw(MException('a:b', 'm'));
catch ME
    disp(ME.message)
end
x = 2;
disp(x)
end
