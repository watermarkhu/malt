x = [];
parfor m = 1:3
    x = [x m];
    x = [m x];
end
