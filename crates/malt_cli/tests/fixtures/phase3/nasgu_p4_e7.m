err = [];
try
    error('a:b', 'Value %d', 42);
catch
end
disp(err)
