classdef ValBase
    % VALBASE A value class with each kind of property and method attribute.
    properties
        Plain double = 1
    end
    properties (Constant)
        Limit = 10
    end
    properties (Dependent)
        Doubled
    end
    properties (Hidden)
        Secret
    end
    properties (Access = private)
        Internal
    end
    properties (SetAccess = protected)
        Guarded
    end
    methods
        function obj = ValBase(v)
            if nargin > 0
                obj.Plain = v;
            end
        end
        function v = get.Doubled(obj)
            v = 2 * obj.Plain;
        end
        function obj = set.Plain(obj, v)
            obj.Plain = v;
        end
    end
    methods (Static)
        function r = make()
            r = ValBase();
        end
    end
    methods (Hidden)
        function r = hiddenHelper(obj)
            r = obj.Plain;
        end
    end
    methods (Access = private)
        function r = privateHelper(obj)
            r = obj.Plain;
        end
    end
end
