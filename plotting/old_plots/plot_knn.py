import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.ticker as ticker

def no_format(value):
    return value
def format_1k(value):
    # Formats large numbers to 'k' notation for thousands
    return f'{int(value/1000)}k' if value >= 1000 else str(int(value))
def format_seconds_minutes(value):
    # Convert seconds to hours and format the number with two decimal places
    hours = value  # Convert seconds to hours
    return f'{hours:.0f}s'  # Format the result as a float with two decimals followed by 'h'

def plot_grouped_data(df, group_column, x_column, y_column, path, titles, x_format, y_format):
    grouped_data = df.groupby(group_column)

    plt.figure(figsize=(10, 6))
    lines = []
    labels = []
    for name, group in sorted(grouped_data, key=lambda x: x[0]):  # Sort by group name
        # Convert runtime from seconds to hours
        line, = plt.plot(group[x_column], group[y_column], marker='o')
        lines.append(line)
        labels.append(name)

    plt.xlabel(titles['x'])
    plt.ylabel(titles['y'])
    plt.title(titles['title'])
    plt.legend(lines, labels)  # Add sorted labels without a title
    plt.grid(True)

    # Set x-axis formatter
    plt.gca().xaxis.set_major_formatter(ticker.FuncFormatter(lambda x, pos: x_format(x)))
    # Set y-axis formatter
    plt.gca().yaxis.set_major_formatter(ticker.FuncFormatter(lambda y, pos: y_format(y)))

    plt.savefig(path)

# Apply the function with direct conversion of runtime to hours
titles = {
    'title': 'K over Runtime for N = 10k',
    'y': 'Runtime (minute)',
    'x': 'K',
}
plot_grouped_data(pd.read_csv('knn.csv'), 'mode', 'k', 'runtime', 'k_runtime.png', titles, no_format, format_seconds_minutes)
plot_grouped_data(
    pd.read_csv('knn.csv'),
    'mode',
    'k',
    'avg_cr',
    'k_avgcr.png',
    {
        'title': 'K over Avg CR for N = 10k',
        'y': 'Compression Ratio',
        'x': 'K',
    },
    no_format,
    no_format,
)

