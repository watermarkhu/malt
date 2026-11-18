classdef (Sealed = false, AllowedSubclasses = {?V2}) V1 < handle
    properties (Abstract = true)
        ab
    end
    properties (WeakHandle = false, Constant = true)
        k = 1
    end
    properties (Constant, Constant = false)
        k2 = 1
    end
    methods (Access = 'private', Abstract = true)
        f1(obj)
    end
    methods (Access = {?V2}, Abstract)
        f2(obj)
    end
    methods (Abstract, Access = private)
        function r = f3(obj)
        end
    end
    methods (Access = protected)
        function validateX(v)
            if v < 0, error('x'); end
        end
        function validateY()
        end
        function validateZ(v)
            q = 3;
        end
        function validateW(v)
            if v < limit, error('x'); end
            mustBeNumeric(v);
            s = struct('a', v.b);
        end
    end
    methods
        function set.ab(obj, v)
        end
    end
    methods (Abstract)
        set.k(obj, v)
        v = get.k2(obj)
    end
end
