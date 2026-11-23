classdef PkgCls
    % PKGCLS A value class in the nested package a.b.
    properties
        Value double = 0
    end
    methods
        function obj = PkgCls(v)
            obj.Value = v;
        end
    end
end
