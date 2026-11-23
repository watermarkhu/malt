classdef l_registry < handle
    %L_REGISTRY Process-wide registry of shape factories.
    %   There is one registry per MATLAB session. It is created on
    %   first use and kept in a persistent variable of INSTANCE.
    %
    %   Example:
    %       l_registry.register("disc", @(r) l_circle(r));
    %       f = l_registry.lookup("disc");
    %       c = f(3);

    properties (SetAccess = private)
        Count (1,1) double = 0
    end

    properties (Access = private)
        Factories containers.Map
    end

    events
        ShapeRegistered
        ShapeRemoved
    end

    methods (Access = private)
        function obj = l_registry()
            obj.Factories = containers.Map('KeyType', 'char', 'ValueType', 'any');
        end
    end

    methods (Static)
        function obj = instance()
            %INSTANCE Return the single registry, creating it if needed.
            persistent singleton
            if isempty(singleton) || ~isvalid(singleton)
                singleton = l_registry();
            end
            obj = singleton;
        end

        function register(name, factory)
            %REGISTER Associate a name with a factory function handle.
            reg = l_registry.instance();
            if ~isa(factory, 'function_handle')
                error('l_registry:notHandle', ...
                    'Factory for %s must be a function handle.', name);
            end
            reg.Factories(name) = factory;
            reg.Count = reg.Factories.Count;
            notify(reg, 'ShapeRegistered');
        end

        function f = lookup(name)
            %LOOKUP Return the factory for name, or [] when unknown.
            reg = l_registry.instance();
            if ~isKey(reg.Factories, name)
                f = [];
                return
            end
            f = reg.Factories(name);
        end

        function names = listNames()
            %LISTNAMES Sorted list of registered names.
            reg = l_registry.instance();
            names = sort(keys(reg.Factories));
        end

        function unregister(name)
            %UNREGISTER Remove a factory if it is present.
            reg = l_registry.instance();
            if isKey(reg.Factories, name)
                remove(reg.Factories, name);
                reg.Count = reg.Factories.Count;
                notify(reg, 'ShapeRemoved');
            end
        end

        function reset()
            %RESET Drop every registration. Intended for test teardown.
            reg = l_registry.instance();
            reg.Factories = containers.Map('KeyType', 'char', 'ValueType', 'any');
            reg.Count = 0;
        end
    end
end
