classdef i_manu_error
    methods
        function obj = fail(obj)
            error('I:fail', 'always fails');
        end

        function obj = keep(obj)
        end

        function obj = fill(obj, v) %#ok<INUSD>
            obj = 1;
        end
    end
end
