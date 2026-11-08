classdef V4 < handle
    properties
        limit = 3
        x
    end
    methods
        function validateInputs(obj, data)
            n = numel(data);
            if n > obj.limit
                error('V4:big', 'too big');
            end
        end
        function validateNone()
        end
        function validateEmpty(obj)
        end
    end
end
