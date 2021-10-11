#!/bin/bash
LINUX_COMMIT=${LINUX_COMMIT:-49b981}
OPENSBI_COMMIT=${OPENSBI_COMMIT:-27d5b5f}

scp ./firesim-scripts/scripts-laputa/generate_firesim_linux_kernel.sh ldj@r641:~/firesim/chipyard/software/firemarshal/

ssh ldj@r641 "cd ~/firesim/chipyard/software/firemarshal/ && LINUX_COMMIT=$LINUX_COMMIT OPENSBI_COMMIT=$OPENSBI_COMMIT ./generate_firesim_linux_kernel.sh"

scp ldj@r641:~/firesim/chipyard/software/firemarshal/images/br-base-bin .

