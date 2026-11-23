classdef ValSub < ValBase
    % VALSUB A value class that inherits a value class.
    properties
        Extra string = "x"
    end
    methods
        function obj = ValSub()
            obj@ValBase(3);
        end
    end
end
