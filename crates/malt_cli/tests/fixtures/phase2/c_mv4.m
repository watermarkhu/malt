classdef c_mv4
    properties
        x = 0
    end
    methods
        function set_x(obj, v)
            for k = 1:2
                disp(obj)
                obj.x = v;
            end
        end
    end
end
