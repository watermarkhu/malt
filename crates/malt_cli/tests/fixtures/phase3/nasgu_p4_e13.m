function nasgu_p4_e13()
err = [];
try
    error('a:b', 'Value %d', 42);
catch err
end
if ~isempty(err)
    disp(err.message)
end
end
