err = [];
try
    error('Value %d', 42);
catch err
end
disp(err)
