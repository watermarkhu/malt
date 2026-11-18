try
    error('a:b', 'x');
catch ME
    warning(ME.message);
end
