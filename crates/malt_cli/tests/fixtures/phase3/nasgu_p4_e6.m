err = [];
try
    error('a:b', 'Value %d', 42);
catch ME
    err = ME;
end
disp(err)
