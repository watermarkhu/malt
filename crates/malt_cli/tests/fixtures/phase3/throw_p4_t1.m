function throw_p4_t1(a)
x = 1;
if a
    throw(MException('a:b', 'm'));
end
x = 2;
disp(x)
end
