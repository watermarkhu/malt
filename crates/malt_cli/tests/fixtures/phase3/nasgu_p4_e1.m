err = [];
try
    error('a:b', 'Value %d', 42);
catch err
end
disp(err)
