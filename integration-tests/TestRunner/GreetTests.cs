using Xunit;

namespace TestRunner
{
    public class GreetTests
    {
        [Fact]
        public void GreetNumber()
        {
            string result = IntegrationTests.GreetANumber(7);
            Assert.Equal("Hello, #7!", result);
        }

        [Fact]
        public void GreetNumberRepeated()
        {
            for (int number = 0; number < 1000; number += 1)
            {
                string actual = IntegrationTests.GreetANumber(number);
                Assert.Equal($"Hello, #{number}!", actual);
            }
        }

        [Fact]
        public void ReturnNumber()
        {
            int result = IntegrationTests.ReturnANumber();
            Assert.Equal(7, result);
        }
    }
}
