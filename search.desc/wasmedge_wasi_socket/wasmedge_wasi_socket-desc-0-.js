searchState.loadedDescShard("wasmedge_wasi_socket", 0, "The size of an IPv4 address in bits.\nThe size of an IPv6 address in bits.\nAn IPv4 address representing the broadcast address: …\nBoth the reading and the writing portions of the <code>TcpStream</code> …\nAn IP address, either IPv4 or IPv6.\nAn IPv4 address.\nAn IPv6 address.\nAn IPv4 address with the address pointing to localhost: …\nAn IPv6 address representing localhost: <code>::1</code>.\nThe reading portion of the <code>TcpStream</code> should be shut down.\nPossible values which can be passed to the …\nAn internet socket address, either IPv4 or IPv6.\nAn IPv4 address representing an unspecified address: …\nAn IPv6 address representing the unspecified address: <code>::</code>.\nAn IPv4 address.\nAn IPv4 socket address.\nAn IPv6 address.\nAn IPv6 socket address.\nThe writing portion of the <code>TcpStream</code> should be shut down.\nAccept incoming connections with given file descriptor …\nCreate TCP socket and bind to the given address.\nCreate UDP socket and bind to the given address.\nCreate TCP socket and connect to the given address.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCreates an <code>IpAddr::V4</code> from a four element byte array.\nCopies this address to a new <code>IpAddr::V6</code>.\nReturns the argument unchanged.\nCopies this address to a new <code>IpAddr::V4</code>.\nCreates an <code>IpAddr::V6</code> from an eight element 16-bit array.\nCreates an <code>IpAddr::V6</code> from a sixteen element byte array.\nReturns the argument unchanged.\nUses <code>Ipv4Addr::from_bits</code> to convert a host byte order <code>u32</code> …\nCreates an <code>Ipv4Addr</code> from a four element byte array.\nCreates an <code>Ipv6Addr</code> from a sixteen element byte array.\nReturns the argument unchanged.\nCreates an <code>Ipv6Addr</code> from an eight element 16-bit array.\nUses <code>Ipv6Addr::from_bits</code> to convert a host byte order <code>u128</code> …\nReturns the argument unchanged.\nConverts a tuple struct (Into&lt;<code>IpAddr</code>&gt;, <code>u16</code>) into a …\nConverts a <code>SocketAddrV6</code> into a <code>SocketAddr::V6</code>.\nConverts a <code>SocketAddrV4</code> into a <code>SocketAddr::V4</code>.\nConverts a native byte order <code>u32</code> into an IPv4 address.\nConverts a native byte order <code>u128</code> into an IPv6 address.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nReturns the IP address associated with this socket address.\nReturns <code>true</code> if this address is in a range designated for …\nReturns <code>true</code> if this address part of the <code>198.18.0.0/15</code> …\nReturns <code>true</code> if this is an address reserved for …\nReturns <code>true</code> if this is a broadcast address (…\nReturns <code>true</code> if this address is in a range designated for …\nReturns <code>true</code> if this address is in a range designated for …\nReturns <code>true</code> if this is an address reserved for …\nReturns <code>true</code> if the address appears to be globally …\nReturns <code>true</code> if the address appears to be globally …\nReturns <code>true</code> if the address appears to be globally …\nReturns <code>true</code> if this address is an <code>IPv4</code> address, and <code>false</code> …\nReturns <code>true</code> if the IP address in this <code>SocketAddr</code> is an …\nReturns <code>true</code> if the address is an IPv4-mapped address (…\nReturns <code>true</code> if this address is an <code>IPv6</code> address, and <code>false</code> …\nReturns <code>true</code> if the IP address in this <code>SocketAddr</code> is an …\nReturns <code>true</code> if the address is link-local (<code>169.254.0.0/16</code>).\nReturns <code>true</code> if this is a loopback address.\nReturns <code>true</code> if this is a loopback address (<code>127.0.0.0/8</code>).\nReturns <code>true</code> if this is the loopback address (<code>::1</code>), as …\nReturns <code>true</code> if this is a multicast address.\nReturns <code>true</code> if this is a multicast address (<code>224.0.0.0/4</code>).\nReturns <code>true</code> if this is a multicast address (<code>ff00::/8</code>).\nReturns <code>true</code> if this is a private address.\nReturns <code>true</code> if this address is reserved by IANA for …\nReturns <code>true</code> if this address is part of the Shared Address …\nReturns <code>true</code> if this is a unicast address, as defined by …\nReturns <code>true</code> if the address is a globally routable unicast …\nReturns <code>true</code> if the address is a unicast address with …\nReturns <code>true</code> if this is a unique local address (<code>fc00::/7</code>).\nReturns <code>true</code> for the special ‘unspecified’ address.\nReturns <code>true</code> for the special ‘unspecified’ address (…\nReturns <code>true</code> for the special ‘unspecified’ address (<code>::</code>…\nGet local address.\nGet local address.\nReturns the address’s multicast scope if the address is …\nCreates a new IPv4 address from four eight-bit octets.\nCreates a new IPv6 address from eight 16-bit segments.\nCreates a new socket address from an IP address and a port …\nReturns the four eight-bit integers that make up this …\nReturns the sixteen eight-bit integers the IPv6 address …\nParse an IP address from a slice of bytes.\nParse an IPv4 address from a slice of bytes.\nParse an IPv6 address from a slice of bytes.\nParse a socket address from a slice of bytes.\nGet peer address.\nReturns the port number associated with this socket …\nReturns the eight 16-bit segments that make up this …\nChanges the IP address associated with this socket address.\nChanges the port number associated with this socket …\nConverts an IPv4 address into a <code>u32</code> representation using …\nConverts an IPv6 address into a <code>u128</code> representation using …\nConverts this address to an <code>IpAddr::V4</code> if it is an …\nConverts this address to an <code>IpAddr::V4</code> if it is an …\nConverts this address to an <code>IPv4</code> address if it is either …\nConverts this address to an <code>IPv4</code> address if it’s an …\nConverts this address to an IPv4-compatible <code>IPv6</code> address.\nConverts this address to an IPv4-mapped <code>IPv6</code> address.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nGet Address Information\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.")