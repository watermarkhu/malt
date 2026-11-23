err = [];
try
    error('a:b', 'msg');
catch err
end
disp(err)
