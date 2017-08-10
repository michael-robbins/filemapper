#!/bin/bash

BIN='../target/debug/file_mapper'
CONF='test_config.yaml'

OUTPUT='/tmp/output.tsv'
EXPECTED_OUTPUT='expected_output.tsv'

cargo build
$BIN --config-file $CONF > /tmp/output.tsv

OUTPUT_HASH=`md5sum ${OUTPUT} | awk '{print $1}'`
EXPECTED_OUTPUT_HASH=`md5sum ${EXPECTED_OUTPUT} | awk '{print $1}'`

echo "NOTICE: Output Hash: ${OUTPUT_HASH}"
echo "NOTICE: Expected Output Hash: ${EXPECTED_OUTPUT_HASH}"

if [ "${OUTPUT_HASH}" != "${EXPECTED_OUTPUT_HASH}" ]; then
    echo "ERROR: Hashes were different!"
    echo ${OUTPUT}
    echo ${EXPECTED_OUTPUT}
    diff ${OUTPUT} ${EXPECTED_OUTPUT}
else
    echo "NOTICE: Hashes match for output:"
    cat ${OUTPUT}
fi

rm ${OUTPUT}
