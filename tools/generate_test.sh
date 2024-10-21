#! /bin/bash
set -e;

if [ ! -d "$1" ]; then
    echo "First arg is not a valid directory";
    exit 1
fi

for dump_file in $(find "${1}" -type f -name *.dev | sort | uniq);
do
    BASE_NAME=$(basename "${dump_file}");
    TEST_NAME=$(echo "$BASE_NAME" | awk -F "__" '{print $1"_"$2"_"$3"_"$4"_v"$5}' | sed "s/\\.\\|-\\|+\\|\"\\|'\\|\`/_/g");
    UPS_NAME=$(echo "$BASE_NAME" | awk -F "__" '{print $1"_"$2}');

    cat <<EOF
    ups_validation_test!(
      test_name = ${TEST_NAME,,},
      dump_file = ".${dump_file}",
      ups_name = "${UPS_NAME}"
  );
EOF

done;
