import json
import sys
import traceback

import xlrd


def parse_excel(book: xlrd.Book):
    # print('The number of worksheets is {0}'.format(book.nsheets))
    # print('Worksheet name(s): {0}'.format(book.sheet_names()))
    sh = book.sheet_by_index(0)
    header = sh.row_values(0)
    # print(header)
    # cols = [[xlrd.xldate_as_datetime(x, book.datemode).date().isoformat()
    # for x in sh2.col_values(0, start_rowx=1) if x]]
    data = {}
    for i, col in enumerate(header):
        data[col] = sh.col_values(i, start_rowx=1)
    return data


if __name__ == '__main__':
    for line in sys.stdin:
        try:
            input_info = json.loads(line)
            file = input_info['file']
            with xlrd.open_workbook(file, logfile=sys.stderr) as book:
                ret = parse_excel(book)
            json.dump(ret, sys.stdout, ensure_ascii=False,
                      separators=(',', ':'))
            print(flush=True)
        except Exception as e:
            _info = traceback.extract_tb(sys.exc_info()[2])[-1]
            print(f'!{_info.filename}:{_info.lineno}:{repr(e)}', flush=True)
