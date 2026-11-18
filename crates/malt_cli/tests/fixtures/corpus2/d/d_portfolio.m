classdef d_portfolio
%D_PORTFOLIO Value-class portfolio of share positions and cash.
%   P = d_portfolio.fromCash(10000) creates an empty portfolio.
%   P = P.buy("AAPL", 10, 150) returns a new portfolio with the position.
%   Value semantics: the original object is unchanged by buy and sell.

    properties
        Holdings table = table('Size', [0 3], ...
            'VariableTypes', {'string', 'double', 'double'}, ...
            'VariableNames', {'Ticker', 'Shares', 'Price'})
        Cash (1,1) double {mustBeNonnegative} = 0
    end

    properties (Constant)
        FeeRate = 0.001;    % 10 bp per trade
    end

    methods
        function obj = d_portfolio(cash)
            % Constructor with optional starting cash
            if nargin > 0
                obj.Cash = cash;
            end
        end

        function obj = buy(obj, ticker, shares, price)
            obj.validateTrade(shares, price);
            cost = shares * price * (1 + obj.FeeRate);
            if cost > obj.Cash
                error('d_portfolio:insufficientCash', ...
                    'Need %.2f but only %.2f cash available.', cost, obj.Cash);
            end
            obj.Cash = obj.Cash - cost;

            idx = find(obj.Holdings.Ticker == string(ticker), 1);
            if isempty(idx)
                obj.Holdings(end+1, :) = {string(ticker), shares, price};
            else
                obj.Holdings.Shares(idx) = obj.Holdings.Shares(idx) + shares;
                obj.Holdings.Price(idx) = price;
            end
        end

        function obj = sell(obj, ticker, shares, price)
            obj.validateTrade(shares, price);
            idx = find(obj.Holdings.Ticker == string(ticker), 1);
            if isempty(idx) || obj.Holdings.Shares(idx) < shares
                error('d_portfolio:noPosition', 'Not enough %s to sell.', ticker);
            end
            proceeds = shares * price * (1 - obj.FeeRate);
            obj.Cash = obj.Cash + proceeds;
            obj.Holdings.Shares(idx) = obj.Holdings.Shares(idx) - shares;

            % Drop positions that are now flat
            obj.Holdings(obj.Holdings.Shares == 0, :) = [];
        end

        function s = toString(obj)
            s = sprintf('Cash: %.2f, positions: %d', obj.Cash, height(obj.Holdings));
        end

        function disp(obj)
            fprintf('%s\n', obj.toString());
            if height(obj.Holdings) > 0
                disp(obj.Holdings);
            end
        end
    end

    methods (Static)
        function obj = fromCash(amount)
            % Factory: an empty portfolio with the given cash
            arguments
                amount (1,1) double {mustBeNonnegative} = 0
            end
            obj = d_portfolio(amount);
        end
    end

    methods (Access = private)
        function validateTrade(~, shares, price)
            if shares <= 0 || price <= 0
                error('d_portfolio:badTrade', 'Shares and price must be positive.');
            end
        end
    end
end
