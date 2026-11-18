classdef ValClass
    properties (SetObservable)
        Data
    end
    properties (Constant, SetAccess = public)
        X = 1
    end
    properties (Constant, GetObservable)
        W
    end
    properties (Constant)
        Z
    end
    properties (Dependent)
        Y = 5
    end
end
