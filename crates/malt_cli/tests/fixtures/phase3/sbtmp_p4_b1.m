classdef sbtmp_p4_b1 < handle
properties
    Subscribers cell = cell(1, 0)
end
methods
    function run(obj, value)
        for k = 1:numel(obj.Subscribers)
            obj.Subscribers{k}(value);
        end
    end
end
end
