// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

import React from 'react';
import { Box, Grid, useColorMode } from '@chakra-ui/react';
import { secondaryBgColor } from 'core/colors';
import ImportWalletHeader from 'core/components/ImportWalletHeader';

interface WalletLayoutProps {
  backPage?: string;
  children: React.ReactNode;
  headerValue?: string;
}

export default function ImportWalletLayout({
  backPage,
  children,
  headerValue = 'Import wallet',
}: WalletLayoutProps) {
  const { colorMode } = useColorMode();

  return (
    <Grid
      height="100%"
      width="100%"
      maxW="100%"
      templateRows="64px 1fr"
      bgColor={secondaryBgColor[colorMode]}
    >
      <ImportWalletHeader backPage={backPage} headerValue={headerValue} />
      <Box maxH="100%" overflowY="auto" pb={4}>
        {children}
      </Box>
    </Grid>
  );
}
