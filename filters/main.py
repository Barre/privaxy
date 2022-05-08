from registry import get_filters, FilterException
import sys
import json


def eprint(*args, **kwargs):
    print(*args, file=sys.stderr, **kwargs)


def main():
    filters = get_filters()

    meta_data = []

    for filter in filters:
        print(f"Processing filter: {filter.title}")

        try:
            filter.save_to_registry()
        except FilterException as e:
            eprint(f"Failed to fetch filter: {e}")

            continue

        meta_data.append(filter.to_dict())

    with open("registry/metadata.json", "w") as f:
        f.write(json.dumps(meta_data, indent=4, sort_keys=True, ensure_ascii=False))


if __name__ == "__main__":
    main()
