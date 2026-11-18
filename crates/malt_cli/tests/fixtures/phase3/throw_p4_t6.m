function throw_p4_t6()
err = [];
try
    throw(MException('a:b', 'm'));
catch err
end
disp(err)
end
