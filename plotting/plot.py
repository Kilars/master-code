import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.ticker as ticker

def no_format(value):
    return value
def format_1k(value):
    # Formats large numbers to 'k' notation for thousands
    return f'{int(value/1000)}k' if value >= 1000 else str(int(value))
def format_seconds_hours(value):
    # Convert seconds to hours and format the number with two decimal places
    hours = value / 3600  # Convert seconds to hours
    return f'{hours:.0f}h'  # Format the result as a float with two decimals followed by 'h'

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
    'title': 'N over Runtime',
    'y': 'Runtime (hours)',
    'x': 'N',
}
plot_grouped_data(pd.read_csv('compression.csv'), 'mode', 'n', 'runtime', 'n_runtime.png', titles, format_1k, format_seconds_hours)
plot_grouped_data(
    pd.read_csv('compression.csv'),
    'mode',
    'n',
    'compression_ratio',
    'n_compression.png',
    {
        'title': 'N over Compression Ratio',
        'y': 'Compression Ratio',
        'x': 'N',
    },
    format_1k,
    no_format,
)
plot_grouped_data(
    pd.read_csv('set_size.csv'),
    'mode',
    'n',
    'set_size',
    'n_set_size.png',
    {
        'title': 'Sample Size over Set Size',
        'y': 'Set Size',
        'x': 'Sample Size',
    },
    format_1k,
    format_1k,
)
plot_grouped_data(
    pd.read_csv('set_size.csv'),
    'mode',
    'n',
    'runtime',
    'n_set_runtime.png',
    {
        'title': 'Set building sample size over Runtime',
        'y': 'Runtime (hours)',
        'x': 'Sample Size',
    },
    format_1k,
    format_seconds_hours,
)
