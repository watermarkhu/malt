classdef c_money
    % c_money is an immutable amount in one currency, in minor units (cents).

    properties (SetAccess = immutable)
        Cents (1,1) int64
        Currency (1,3) char {mustBeUpper}
    end

    properties (Dependent)
        Amount
    end

    methods
        function obj = c_money(cents, currency)
            arguments
                cents (1,1) {mustBeInteger}
                currency (1,3) char = 'EUR'
            end
            obj.Cents = int64(cents);
            obj.Currency = currency;
        end

        function a = get.Amount(obj)
            a = double(obj.Cents) / 100;
        end

        function m = plus(a, b)
            c_money.checkSame(a, b);
            m = c_money(a.Cents + b.Cents, a.Currency);
        end

        function m = minus(a, b)
            c_money.checkSame(a, b);
            m = c_money(a.Cents - b.Cents, a.Currency);
        end

        function m = times(a, k)
            m = c_money(round(double(a.Cents) * k), a.Currency);
        end

        function tf = lt(a, b)
            c_money.checkSame(a, b);
            tf = a.Cents < b.Cents;
        end

        function tf = eq(a, b)
            tf = isa(b, 'c_money') && a.Cents == b.Cents && strcmp(a.Currency, b.Currency);
        end

        function s = char(obj)
            s = sprintf('%.2f %s', obj.Amount, obj.Currency);
        end
    end

    methods (Static, Access = private)
        function checkSame(a, b)
            if ~strcmp(a.Currency, b.Currency)
                error('c_money:currency', 'Mixed currencies %s and %s', a.Currency, b.Currency);
            end
        end
    end
end
