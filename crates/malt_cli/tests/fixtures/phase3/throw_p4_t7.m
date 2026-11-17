function throw_p4_t7()
x = 1;
try
    throw(MException('a:b', 'm'));
catch x
end
disp(x)
end
