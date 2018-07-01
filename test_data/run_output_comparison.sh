#!/bin/bash

BIN='../target/debug/file_mapper'
CONF='test_config.yaml'

ACTUAL_OUTPUT='/tmp/output.tsv'
EXPECT_OUTPUT='expected_output.tsv'

# Compile any changes and perform a mapping run
cargo build
$BIN --config-file $CONF > /tmp/output.tsv

# Generate the hashes
ACTUAL_HASH=`md5sum ${ACTUAL_OUTPUT} | awk '{print $1}'`
EXPECT_HASH=`md5sum ${EXPECT_OUTPUT} | awk '{print $1}'`

echo "NOTICE: Actual Hash: ${ACTUAL_HASH}"
echo "NOTICE: Expect Hash: ${EXPECT_HASH}"

# Report on outcome
if [ "${ACTUAL_HASH}" != "${EXPECT_HASH}" ]; then
    echo "ERROR: Hashes were different!"
    echo ${ACTUAL_OUTPUT}
    echo ${EXPECT_OUTPUT}
    diff ${ACTUAL_OUTPUT} ${EXPECT_OUTPUT}
else
    echo "NOTICE: Hashes match for output:"
    cat ${ACTUAL_OUTPUT}
fi

rm ${ACTUAL_OUTPUT}

