classdef a_getset_dependent
    % Dependent properties with their accessors, and a private store.

    properties (Dependent)
        Entries (1,1) double
        Level (1,1) string
    end

    properties (Access = private)
        Proxy
        Levels = ["none" "debug"]
    end

    methods
        function obj = a_getset_dependent()
            obj.Proxy = 1;
        end

        function entries = get.Entries(obj)
            entries = obj.Proxy;
        end

        function obj = set.Entries(obj, entries)
            obj.Proxy = entries;
        end

        function level = get.Level(obj)
            level = obj.Levels(obj.Proxy);
        end

        function set.Level(obj, level)
            obj.Proxy = find(obj.Levels == level);
        end
    end
end
