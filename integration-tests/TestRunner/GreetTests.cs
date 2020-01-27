using System;
using Xunit;

namespace TestRunner
{
    public class GreetTests
    {
        [Fact]
        public void Test1()
        {
            var result = IntegrationTests.GreetANumber(7);
            Assert.Equal("Hello, #7!", result);
        }
    }
}
